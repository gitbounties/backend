mod github;

use std::env;

use axum::{
    extract::{Json, Path, Query},
    response::Html,
    routing::{get, post},
    Router,
};
use log::{debug, info, warn};
use serde_json::json;

use crate::models::Issue;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/register", get(register))
        .merge(github::router())
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
