use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create))
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    /// Value of the reward
    pub bounty: u64,
    pub owner: String,
    pub repo: String,
    pub issue: u64,
}

/// Create a new issue given URL
pub async fn create(State(state): State<AppState>, Json(payload): Json<CreateBody>) {

    // NOTE shoud we check that the user is owner of the issue to monetize it?

    // fetch info about the issue

    // Open issue as new bounty

    // Send notification on the original issue to mark it as a bounty
}
