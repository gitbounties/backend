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
        .merge(github::router())
}

async fn health() -> &'static str {
    "health!"
}
