use axum::{
    extract::{Json, Path, Query},
    routing::{get, post},
    Router,
};
use log::{debug, info, warn};

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    // build our application with a single route
    let app = Router::new()
        .route("/health", get(health))
        .route("/github/hook", post(github_webhook));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "health!"
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
