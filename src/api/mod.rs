mod bounty;
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

use crate::{models::Issue, AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/github", github::router())
        .nest("/bounty", bounty::router())
}

async fn health() -> &'static str {
    "health!"
}
