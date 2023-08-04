use std::{collections::HashMap, env};

use axum::{
    extract::{Json, Path, Query},
    response::Html,
    routing::{get, post},
    Router,
};
use log::{debug, info, warn};

mod db;

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    dotenv::dotenv().ok();

    db::connect("127.0.0.1:8000").await.unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/health", get(health))
        .route("/register", get(register))
        .route("/github/hook", post(github_webhook))
        .route("/github/callback", get(github_callback));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "health!"
}

async fn register() -> Html<String> {
    // TODO maybe move this so we aren't always reading env var
    let client_id = env::var("CLIENT_ID").expect("Unable to get CLIENT_ID env var");

    Html(format!(
        r#"<a href="https://github.com/login/oauth/authorize?client_id={}">Register with GitHub</a>"#,
        client_id
    ))
}

#[derive(Debug)]
struct Issue {
    title: String,
    body: String,
    url: String,
    node_id: String,
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
    debug!("registered with github {:?}", code);
}
