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
use log::{debug, error, info, warn};
use serde_json::json;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

use crate::{
    db::DBConnection,
    models::{Issue, User},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", get(github_register))
        .route("/hook", post(github_webhook))
        .route("/callback", get(github_callback))
}
async fn github_register() -> Html<String> {
    // TODO maybe move this so we aren't always reading env var
    let client_id = env::var("CLIENT_ID").expect("Unable to get CLIENT_ID env var");

    Html(format!(
        r#"<a href="https://github.com/login/oauth/authorize?client_id={}">Register with GitHub</a>"#,
        client_id
    ))
}

async fn github_webhook(Json(payload): Json<serde_json::Value>) {
    // TODO return proper error to sender
    let action: &str = payload["action"].as_str().expect("Malformed webhook");
    info!("github hook called: {}", action);
    match action {
        "opened" => {
            let issue_raw: &serde_json::Value = payload.get("issue").expect("No issue field");

            let issue = Issue {
                title: issue_raw["title"].as_str().unwrap().into(),
                body: issue_raw["body"].as_str().unwrap().into(),
                url: issue_raw["html_url"].as_str().unwrap().into(),
                node_id: issue_raw["node_id"].as_str().unwrap().into(),
            };

            debug!("[webhook] issue opened {:?}", issue);
        },
        _ => {
            warn!("Unhandled action type {}", action);
        },
    }
}

async fn github_callback(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) {
    let code = params.get("code").expect("code not provided");

    debug!("registered with github {:?}", code);

    let access_token = get_user_access_token(&state.reqwest, code).await.unwrap();

    let profile = get_user_profile(&state.reqwest, &access_token)
        .await
        .unwrap();

    // register user if not in db
    let res: User = state
        .db_conn
        .create("Users")
        .content(User {
            username: profile["login"].as_str().unwrap().into(),
        })
        .await
        .unwrap();
}

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
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(auth)
        .send()
        .await
        .unwrap();

    println!("{:?}", res.text().await.unwrap());

    todo!()
    /*
    let body = res
        .json::<serde_json::Value>()
        .await?;

    debug!("User profile {res:?}");

    Ok(res)
    */
}
