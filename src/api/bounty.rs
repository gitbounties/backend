use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use log::debug;
use serde::Deserialize;

use super::github::get_installation_access_token;
use crate::{
    models::{Bounty, Issue},
    session_auth::{AuthUser, MyRequireAuthorizationLayer},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/",
        post(create).layer(MyRequireAuthorizationLayer::login()),
    )
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    /// Value of the reward
    pub reward: u64,
    pub owner: String,
    pub repo: String,
    pub issue: u64,
}

/// Create a new issue given URL
pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    // Json(payload): Json<CreateBody>,
) -> impl IntoResponse {
    // NOTE shoud we check that the user is owner of the issue to monetize it?

    format!("Logged in as: {}", user.id)
    /*
    //debug!("jwt {}", state.github_jwt);

    // auth process as referenced here
    // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation

    // get the installation id
    let installation_access_token =
        get_installation_access_token(&state, &payload.owner, &payload.repo).await;

    // Check if user has permission to manage this installation

    // fetch info about the issue
    let res = state
        .reqwest
        .get(&format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            payload.owner, payload.repo, payload.issue
        ))
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(installation_access_token)
        .send()
        .await
        .unwrap();

    if !res.status().is_success() {
        let body = res.text().await.unwrap();
        // TODO return proper response
        panic!("Issue does not exist {}", body);
    }

    let body = res.json::<serde_json::Value>().await.unwrap();

    // debug!("issue {}", body);

    // Open issue as new bounty
    // TODO throw warning if already registered
    let res: Bounty = state
        .db_conn
        .create("Bounty")
        .content(Bounty {
            reward: payload.reward,
            owner: String::new(), // TODO
            issue: Issue {
                owner: payload.owner,
                repo: payload.repo,
                issue_id: body["id"].as_u64().unwrap() as usize,
            },
        })
        .await
        .unwrap();

    // generate smart contract

    // Send notification on the original issue to mark it as a bounty
    */
}
