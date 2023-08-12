//! Github specific routes
//!
//!

use std::{collections::HashMap, env, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post, MethodRouter},
    Router,
};
use axum_login::{axum_sessions::async_session::MemoryStore, extractors::AuthContext};
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::json;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

use crate::{
    db::DBConnection,
    models::User,
    session_auth::{AuthUser, MyAuthContext},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", get(github_register))
        .route("/dummy/login", post(dummy_login))
        .route("/hook", post(github_webhook))
        .route("/callback", get(github_callback))
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
            issue_closed_webhook(&state).await;
        },
        _ => {
            warn!("Unhandled action type {}", action);
        },
    }
}

pub(crate) async fn issue_closed_webhook(state: &AppState) {
    // Check to see if issue has an associated bounty, retrieve owner, repo and issue_id
    let owner = "gitbounties";
    let repo = "sample_repo";
    let issue_id = 3;
    let installation_access_token = get_installation_access_token(&state, &owner, &repo).await;

    // TODO Double check that an issue (not PR) is being closed

    // Find the PR that closed this issue
    // TODO find a nicer way to write graphql queries in rust
    let query = format!(
        r#" {{ repository(name: \"{repo}\", owner: \"{owner}\") {{ issue(number: {issue_id}) {{ timelineItems(itemTypes: CLOSED_EVENT, last: 1) {{ nodes {{ ... on ClosedEvent {{ createdAt closer {{ __typename ... on PullRequest {{ author {{ login }} }} }} }} }} }} }} }} }} "#
    );
    println!("{query}");
    let res = state
        .reqwest
        .post("https://api.github.com/graphql")
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(installation_access_token)
        .body(format!(r#"{{ "query": "{query}" }}"#))
        .send()
        .await
        .unwrap();

    // let body = res.json::<serde_json::Value>().await.unwrap();
    let body = res.text().await.unwrap();

    // Find the user the closed this issue and transfer them the funds

    println!("[webhook] issue closed {body}");
}

async fn github_callback(
    Query(params): Query<HashMap<String, String>>,
    mut auth: MyAuthContext,
    State(state): State<AppState>,
) {
    let code = params.get("code").expect("code not provided");

    debug!("auth with github {:?}", params);

    let access_token = get_user_access_token(&state.reqwest, code).await.unwrap();

    let profile = get_user_profile(&state.reqwest, &access_token)
        .await
        .unwrap();

    let username = profile["login"].as_str().unwrap().to_string();

    // register user if not in db
    let res: Option<User> = state.db_conn.select(("Users", &username)).await.unwrap();

    debug!("select res {:?}", res);
    if let Some(user) = res {
        debug!("Logging in user");
        login_user().await;
    } else {
        debug!("Registering new user");
        register_user(&state, &username, &access_token).await;
    }
    auth.login(&AuthUser {
        id: String::from(&username),
    })
    .await
    .unwrap();
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

async fn register_user(state: &AppState, username: &str, access_token: &str) {
    // find installations the user has access to
    let res = state
        .reqwest
        // TODO not safe to simply do string format with user controlled input, should definitely santized payload first
        .get(&format!("https://api.github.com/user/installations",))
        // TODO move user agent to common static constant string
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(&access_token)
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

    let res: User = state
        .db_conn
        .create(("Users", username))
        .content(User {
            username: username.to_string(),
            github_installations: installation_ids,
        })
        .await
        .unwrap();

    debug!("registered user res {res:?}");
}

async fn login_user() {}

/// Exchange code recieved from github callback for a github access token
async fn get_user_access_token(reqwest: &reqwest::Client, code: &str) -> reqwest::Result<String> {
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

    info!("res {:?}", body);

    let access_token = body["access_token"].as_str().unwrap();

    debug!("Recieved access token {access_token}");

    Ok(access_token.into())
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

pub async fn get_installation_access_token(state: &AppState, owner: &str, repo: &str) -> String {
    let res = state
        .reqwest
        // TODO not safe to simply do string format with user controlled input, should definitely santized payload first
        .get(&format!(
            "https://api.github.com/repos/{owner}/{repo}/installation",
        ))
        // TODO move user agent to common static constant string
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(&state.github_jwt)
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();
    debug!("body {body}");
    let installation_id = body["id"].as_u64().expect("Couldn't get installation id");

    // debug!("installation id {installation_id}");

    // get access token of installation
    let res = state
        .reqwest
        .post(&format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        ))
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(&state.github_jwt)
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
    use crate::{api::github::issue_closed_webhook, AppState};

    #[tokio::test]
    async fn test_issue_closed_webhook() {
        dotenvy::dotenv().unwrap();
        let app_state = AppState::init().await;
        issue_closed_webhook(&app_state).await;
    }

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
}
