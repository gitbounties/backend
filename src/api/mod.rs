pub mod auth;
pub mod bounty;
pub mod github;
pub mod issue;

use std::env;

use axum::{
    extract::{Json, Path, Query},
    response::Html,
    routing::{get, post},
    Router,
};
use log::{debug, info, warn};
use serde_json::json;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/dev", get(dev))
        .nest("/github", github::router())
        .nest("/bounty", bounty::router())
        .nest("/auth", auth::router())
        .nest("/issue", issue::router())
}

async fn health() -> &'static str {
    "health!"
}

/// Temp route to test code
async fn dev() {}
