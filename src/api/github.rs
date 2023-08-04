//! Github specific routes
//!
//!

use std::{collections::HashMap, env};

use axum::{
    extract::{Json, Path, Query},
    response::Html,
    routing::{get, post, MethodRouter},
    Router,
};
use log::{debug, info, warn};
use serde_json::json;

use crate::models::Issue;

pub fn router() -> Router {
    Router::new()
        .route("/github/register", get(github_register))
        .route("/github/hook", post(github_webhook))
        .route("/github/callback", get(github_callback))
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

async fn github_callback(Query(params): Query<HashMap<String, String>>) {
    let code = params.get("code").expect("code not provided");
    get_user_access_token(code).await.unwrap();
    debug!("registered with github {:?}", code);
}

/// Exchange code recieved from github callback for a github access token
async fn get_user_access_token(code: &str) -> reqwest::Result<String> {
    /// TODO not sure if making client each request is slow, could make this static (or shared)?
    let client = reqwest::Client::new();

    let res = client
        .post("https://github.com/login/oauth/access_token")
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
        .await?
        .json::<serde_json::Value>()
        .await?;

    let access_token = res["access_token"].as_str().unwrap();

    debug!("Recieved access token {access_token}");

    Ok(access_token.into())
}

// TODO technically not a route, should move somewhere else?
/// Grab information from user's github profile
async fn get_user_profile(auth: &str) -> reqwest::Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/user")
        .bearer_auth(auth)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    debug!("User profile {res:?}");

    Ok(())
}
