use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use log::debug;
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

    debug!("jwt {}", state.github_jwt);

    // auth process as referenced here
    // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation

    // get the installation id
    let res = state
        .reqwest
        // TODO not safe to simply do string format with user controlled input, should definitely santized payload first
        .get(&format!(
            "https://api.github.com/repos/{}/{}/installation",
            payload.owner, payload.repo
        ))
        // TODO move user agent to common static constant string
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(&state.github_jwt)
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();
    let installation_id = body["id"].as_str().expect("Couldn't get installation id");

    debug!("installation id {installation_id}");

    // get access token of installation
    let res = state
        .reqwest
        .post(&format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        ))
        .header("User-Agent", "GitBounties")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(&state.github_jwt)
        .send()
        .await
        .unwrap();

    let body = res.json::<serde_json::Value>().await.unwrap();
    let installation_access_token = body["token"]
        .as_str()
        .expect("Couldn't get installation access token");

    debug!("installation access token {installation_access_token}");

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

    debug!("issue {}", body);

    // Open issue as new bounty

    // Send notification on the original issue to mark it as a bounty
}
