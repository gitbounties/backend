//! Github specific routes
//!
//!

use std::{collections::HashMap, env, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{get, post, MethodRouter},
    Router,
};
use axum_login::{axum_sessions::async_session::MemoryStore, extractors::AuthContext};
use gitbounties_contract::{get_contract, http_provider, parse_address, Middleware};
use log::{debug, error, info, warn};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde_json::json;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

use crate::{
    db::DBConnection,
    models::{Address, Bounty, BountyStatus, Issue, User},
    session_auth::{AuthUser, MyAuthContext},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", get(github_register))
        .route("/dummy/login", post(dummy_login))
        .route("/hook", post(github_webhook))
        .route("/callback/install", get(github_callback_install))
        .route("/callback/register", get(github_callback_register))
        .route("/callback/login", get(github_callback_login))
    // // NOTE temp endpoint to get access tokens for testing
    // .route("/access_token", get())
}

// TODO we don't actually want to use github register.
// After the app is installed, we should redirect to a sign in page on webapp to link the
// installation with a given GitBounties account
async fn github_register() -> Html<String> {
    // TODO maybe move this so we aren't always reading env var
    let client_id = env::var("CLIENT_ID").expect("Unable to get CLIENT_ID env var");

    Html(format!(
        r#"<a href="https://github.com/login/oauth/authorize?client_id={}">Register with GitHub</a>"#,
        client_id
    ))
}

async fn github_webhook(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) {
    // TODO return proper error to sender
    let action: &str = payload["action"].as_str().expect("Malformed webhook");
    info!("github hook called: {}", action);
    match action {
        "opened" => {
            // NOTE we probably don't actually care when a PR was opened
            // TODO PRs also count as issues, make sure, you can check existence of issue field to
            // make sure
            let issue_raw: &serde_json::Value = payload.get("issue").expect("No issue field");

            debug!("[webhook] issue opened {issue_raw}");
        },
        "closed" => {
            let issue_raw: &serde_json::Value = payload.get("issue").expect("No issue field");

            issue_closed_webhook(&state, issue_raw).await;
        },
        _ => {
            warn!("Unhandled action type {}", action);
        },
    }
}

pub async fn issue_closed_webhook(state: &AppState, payload: &serde_json::Value) {
    // Check to see if issue has an associated bounty, retrieve owner, repo and issue_id
    let html_url = payload["html_url"].as_str().expect("Couldn't get html url");
    let issue = parse_github_url(html_url);

    debug!("issue raw {:?}", payload);

    let installation_id = get_installation(&state, &issue.owner, &issue.repo)
        .await
        .unwrap();
    let installation_access_token = get_installation_access_token(&state, installation_id).await;

    // Check if issue has a bounty open (and that it's not closed)
    let mut res = state
        .db_conn
        .query("SELECT * FROM Bounty WHERE issue == $issue AND status = 'Open")
        .bind(("issue", &issue))
        .await
        .unwrap();

    debug!("issue res {:?}", res);
    let Ok(Some(bounty)) = res.take::<Option<Bounty>>(0) else {
        debug!("Could not find associated bounty");
        return;
    };

    if bounty.status != BountyStatus::Open {
        warn!(
            "Bounty has already been closed {}/{}/{}",
            bounty.issue.owner, bounty.issue.repo, bounty.issue.issue_id
        );
        return;
    }

    // Find the PR that closed this issue
    // TODO find a nicer way to write graphql queries in rust
    let query = format!(
        r#" {{ repository(name: \"{}\", owner: \"{}\") {{ issue(number: {}) {{ timelineItems(itemTypes: CLOSED_EVENT, last: 1) {{ nodes {{ ... on ClosedEvent {{ createdAt closer {{ __typename ... on PullRequest {{ author {{ login }} }} }} }} }} }} }} }} }} "#,
        issue.owner, issue.repo, issue.issue_id
    );

    let res = state
        .reqwest_github(
            Method::POST,
            "https://api.github.com/graphql",
            &installation_access_token,
        )
        .body(format!(r#"{{ "query": "{query}" }}"#))
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();
    //let body = res.text().await.unwrap();

    debug!("timeline res {:?}", body);

    // Find the user the closed this issue and transfer them the funds
    let timeline_nodes = body["data"]["repository"]["issue"]["timeLineItems"]["nodes"]
        .as_array()
        .expect("Couldn't get timeline nodes");
    let closer = &timeline_nodes.get(0).unwrap()["closer"]; // TODO should this be first node or last

    let closer_type = closer["__typename"]
        .as_str()
        .expect("Couldn't get closer type");
    let closer_user = closer["author"]["login"]
        .as_str()
        .expect("Couldn't get closer user");
    if closer_type != "PullRequest" {
        debug!("Issue was not closed by pull request");
        return;
    }

    debug!("Got closer user {closer_user}");

    // Get the closer users's public key
    let closer_user_data: User = state
        .db_conn
        .select(("Users", closer_user))
        .await
        .expect("User should exist in database");

    // Get the token id for the bounty
    let token_id = bounty.token_id;

    // fetch closer wallet address
    resolve_bounty(token_id, &closer_user_data.wallet_address).await;

    // mark the issue as resolved

    println!("[webhook] issue closed {body}");
}

pub(crate) async fn resolve_bounty(token_id: u64, closer_wallet_address: &Address) {
    let provider = http_provider();
    let contract_addr = env::var("CONTRACT_ADDRESS").expect("Couldnt get CONTRACT_ADDRESS env var");
    let wallet_private_key =
        env::var("WALLET_PRIVATE_KEY").expect("Couldnt get WALLET_PRIVATE_KEY env var");
    let contract = get_contract(&provider, &contract_addr, &wallet_private_key)
        .await
        .expect("Coudln't initalize contract");

    let target_balance = provider
        .get_balance(*closer_wallet_address, None)
        .await
        .unwrap();
    debug!(
        "target wallet balance before transaction {}",
        target_balance
    );

    let _reciept = contract
        .transfer_token(token_id.into(), *closer_wallet_address)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    let _reciept = contract
        .burn(token_id.into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    let target_balance = provider
        .get_balance(*closer_wallet_address, None)
        .await
        .unwrap();
    debug!("target wallet balance after transaction {}", target_balance);
}

/// Parses github url to fetch issue info
fn parse_github_url(url: &str) -> Issue {
    use regex::Regex;
    // TODO tiny bit sus method of parsing the html url to get info
    // TODO could cache using lazy static
    let re = Regex::new(r#"https://github.com/(?<owner>.)/(?<repo>.)/issues/()?<issue>."#).unwrap();
    let caps = re.captures(url).unwrap();

    Issue {
        owner: caps["owner"].into(),
        repo: caps["repo"].into(),
        issue_id: caps["issue"].parse::<usize>().unwrap(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterQuery {
    code: String,
    wallet_address: String,
}

#[derive(Debug, Deserialize)]
pub struct CodeQuery {
    code: String,
}

/// Callback when installing the github app
async fn github_callback_install(
    Query(params): Query<RegisterQuery>,
    mut auth: MyAuthContext,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(access_token) = get_user_access_token(&state.reqwest, &params.code).await else {
        return (StatusCode::FORBIDDEN, "invalid access token");
    };

    let profile = get_user_profile(&state.reqwest, &access_token)
        .await
        .unwrap();

    let username = profile["login"].as_str().unwrap().to_string();

    // register user if not in db
    let res: Option<User> = state.db_conn.select(("Users", &username)).await.unwrap();

    if res.is_none() {
        // register user if not exist

        let Ok(wallet_address) = parse_address(&params.wallet_address) else {
            warn!(
                "failed to decode wallet_address {}",
                &params.wallet_address.trim_start_matches("0x")
            );
            return (StatusCode::BAD_REQUEST, "invalid wallet address");
        };
        register_user(&state, &username, &access_token, &wallet_address).await;

        // register_user(&state, &username, &access_token, &wallet_address).await;
    } else {
        // otherwise update installation
        update_user_installations(&state, &username, &access_token).await;
    }

    auth.login(&AuthUser {
        id: String::from(&username),
    })
    .await
    .unwrap();

    (StatusCode::OK, "ok")
}

/// Callback when registering from webapp
async fn github_callback_register(
    Query(params): Query<RegisterQuery>,
    mut auth: MyAuthContext,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(access_token) = get_user_access_token(&state.reqwest, &params.code).await else {
        return (StatusCode::FORBIDDEN, "invalid access token");
    };

    let profile = get_user_profile(&state.reqwest, &access_token)
        .await
        .unwrap();

    let username = profile["login"].as_str().unwrap().to_string();

    // register user if not in db
    let res: Option<User> = state.db_conn.select(("Users", &username)).await.unwrap();

    if res.is_some() {
        return (StatusCode::CONFLICT, "user already exists");
    }

    let Ok(wallet_address) = parse_address(&params.wallet_address) else {
        warn!(
            "failed to decode wallet_address {}",
            &params.wallet_address.trim_start_matches("0x")
        );
        return (StatusCode::BAD_REQUEST, "invalid wallet address");
    };
    register_user(&state, &username, &access_token, &wallet_address).await;
    auth.login(&AuthUser {
        id: String::from(&username),
    })
    .await
    .unwrap();

    (StatusCode::OK, "ok")
}

/// Callback when logging in
async fn github_callback_login(
    Query(params): Query<CodeQuery>,
    mut auth: MyAuthContext,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(access_token) = get_user_access_token(&state.reqwest, &params.code).await else {
        return (StatusCode::FORBIDDEN, "invalid access token");
    };

    let profile = get_user_profile(&state.reqwest, &access_token)
        .await
        .unwrap();

    let username = profile["login"].as_str().unwrap().to_string();

    // Check if user has been registered
    let res: Option<User> = state.db_conn.select(("Users", &username)).await.unwrap();

    if res.is_none() {
        return (StatusCode::NOT_FOUND, "user does not exist");
    }

    auth.login(&AuthUser {
        id: String::from(&username),
    })
    .await
    .unwrap();

    (StatusCode::OK, "ok")
}

async fn register_user(
    state: &AppState,
    username: &str,
    access_token: &str,
    wallet_address: &Address,
) {
    let res: User = state
        .db_conn
        .create(("Users", username))
        .content(User {
            username: username.to_string(),
            github_installations: vec![],
            wallet_address: *wallet_address,
        })
        .await
        .unwrap();

    debug!("registered user res {res:?}");

    update_user_installations(state, username, access_token).await;
}

async fn update_user_installations(state: &AppState, username: &str, access_token: &str) {
    let res = state
        .reqwest_github(
            Method::GET,
            "https://api.github.com/user/installations",
            &access_token,
        )
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();
    debug!("user installations {body}");

    let installations = body["installations"]
        .as_array()
        .expect("Couldn't get installations field");
    let installation_ids = installations
        .iter()
        .map(|installation| installation["id"].as_u64().unwrap() as usize)
        .collect::<Vec<_>>();

    let res = state
        .db_conn
        .query(format!(
            "UPDATE Users:{username} SET github_installations = $installations"
        ))
        .bind(("installations", installation_ids))
        .await
        .unwrap();

    debug!("update user installation {res:?}");
}

#[derive(Debug, Deserialize)]
pub struct DummyLoginBody {
    pub username: String,
}
/// Login system used for testing
async fn dummy_login(mut auth: MyAuthContext, Json(payload): Json<DummyLoginBody>) {
    debug!("dummy login for {}", payload.username);
    auth.login(&AuthUser {
        id: String::from(payload.username),
    })
    .await
    .unwrap();
}

/// Exchange code recieved from github callback for a github access token
async fn get_user_access_token(reqwest: &reqwest::Client, code: &str) -> Option<String> {
    let res = reqwest
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .query(&[
            (
                "client_id",
                env::var("CLIENT_ID").expect("Could not get CLIENT_ID env var"),
            ),
            (
                "client_secret",
                env::var("CLIENT_SECRET").expect("Could not get CLIENT_SECRET env var"),
            ),
            ("code", code.into()),
        ])
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();

    let Some(access_token) = body["access_token"].as_str() else {
        warn!("{:?}", body);
        return None;
    };

    debug!("Recieved access token {access_token}");

    Some(access_token.into())
}

// TODO technically not a route, should move somewhere else?
/// Grab information from user's github profile
async fn get_user_profile(
    reqwest: &reqwest::Client,
    auth: &str,
) -> reqwest::Result<serde_json::Value> {
    let res = reqwest
        .get("https://api.github.com/user")
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(auth)
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await?;

    debug!("User profile {body:?}");

    Ok(body)
}

pub async fn get_installation(state: &AppState, owner: &str, repo: &str) -> Option<u64> {
    let res = state
        .reqwest_github(
            Method::GET,
            &format!("https://api.github.com/repos/{owner}/{repo}/installation"),
            &state.github_jwt,
        )
        .send()
        .await
        .unwrap();

    if !res.status().is_success() {
        return None;
    }

    let body = res.json::<serde_json::Value>().await.unwrap();
    debug!("body {body}");
    let installation_id = body["id"].as_u64().expect("Couldn't get installation id");

    // debug!("installation id {installation_id}");

    Some(installation_id)
}

pub async fn get_installation_access_token(state: &AppState, installation_id: u64) -> String {
    // get access token of installation
    let res = state
        .reqwest_github(
            Method::POST,
            &format!(
                "https://api.github.com/app/installations/{}/access_tokens",
                installation_id
            ),
            &state.github_jwt,
        )
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();
    let installation_access_token = body["token"]
        .as_str()
        .expect("Couldn't get installation access token");

    debug!("installation access token {installation_access_token}");

    installation_access_token.to_owned()
}

#[cfg(test)]
mod tests {
    use gitbounties_contract::H160;

    use crate::{
        api::github::{issue_closed_webhook, resolve_bounty},
        AppState,
    };

    // #[tokio::test]
    // async fn test_issue_closed_webhook() {
    //     dotenvy::dotenv().unwrap();
    //     let app_state = AppState::init().await;
    //     issue_closed_webhook(&app_state).await;
    // }

    /// TEMP: Testing surreal select statements in rust
    #[tokio::test]
    async fn test_select_user() {
        dotenvy::dotenv().unwrap();
        let app_state = AppState::init().await;

        let username = "bill";
        let res = app_state
            .db_conn
            .query("SELECT * FROM Users WHERE username == $username")
            .bind(("username", &username))
            .await
            .unwrap();

        println!("res {res:?}");
    }

    #[tokio::test]
    async fn test_resolved_bounty() {
        dotenvy::dotenv().unwrap();

        let token_id = 1;
        let closer_wallet_address: H160 = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
            .parse()
            .unwrap();
        resolve_bounty(token_id, &closer_wallet_address).await;
    }
}
