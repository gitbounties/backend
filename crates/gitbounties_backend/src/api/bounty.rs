use std::collections::HashMap;

use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use log::debug;
use reqwest::{Method, StatusCode};
use serde::Deserialize;

use super::github::{get_installation, get_installation_access_token};
use crate::{
    models::{Bounty, BountyStatus, Issue, User},
    session_auth::{AuthUser, MyRequireAuthorizationLayer},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create).layer(MyRequireAuthorizationLayer::login()),
        )
        .route("/", get(list).layer(MyRequireAuthorizationLayer::login()))
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    /// Value of the reward
    pub reward: u64,
    pub token_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct IssueQuery {
    pub owner: String,
    pub repo: String,
    pub issue: u64,
}

/// Create a new issue given URL
pub async fn create(
    State(state): State<AppState>,
    query: Query<IssueQuery>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateBody>,
) -> (StatusCode, String) {
    // NOTE shoud we check that the user is owner of the issue to monetize it?

    //debug!("jwt {}", state.github_jwt);

    // auth process as referenced here
    // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation

    // get the installation id
    let Some(installation_id) = get_installation(&state, &query.owner, &query.repo).await else {
        return (StatusCode::NOT_FOUND, "Invalid issue".into());
    };
    let installation_access_token = get_installation_access_token(&state, installation_id).await;

    // Check if user has permission to manage this installation
    let user_data: User = state
        .db_conn
        .select(("Users", &auth_user.id))
        .await
        .expect("User should exist in database");

    debug!(
        "checking if {} installations {:?}",
        installation_id, user_data.github_installations
    );

    if !user_data
        .github_installations
        .contains(&(installation_id as usize))
    {
        return (
            StatusCode::FORBIDDEN,
            "No permission to manage installation".into(),
        );
    }

    // fetch info about the issue
    // TODO convert to graphql?
    let res = state
        .reqwest_github(
            Method::GET,
            &format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                query.owner, query.repo, query.issue
            ),
            &installation_access_token,
        )
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
            user: auth_user.id,
            reward: payload.reward,
            status: BountyStatus::Open,
            issue: Issue {
                owner: query.owner.clone(),
                repo: query.repo.clone(),
                issue_id: body["number"].as_u64().unwrap() as usize,
            },
            title: body["title"].as_str().unwrap().into(),
            description: body["body"].as_str().unwrap().into(), // TODO body may not exist
            labels: body["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|label| label["name"].as_str().unwrap().to_string())
                .collect(),
            created: chrono::offset::Utc::now(),
            token_id: payload.token_id,
        })
        .await
        .unwrap();

    // generate smart contract

    // Send notification on the original issue to mark it as a bounty

    (StatusCode::OK, "Ok".into())
}

/// Get all created bounties for user
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Json<Vec<Bounty>> {
    let mut res = if params.get("user") == Some(&String::from("true")) {
        state
            .db_conn
            .query("SELECT * FROM Bounty WHERE user == $user")
            .bind(("user", auth_user.id))
            .await
            .unwrap()
    } else {
        state.db_conn.query("SELECT * FROM Bounty").await.unwrap()
    };

    let bounties: Vec<Bounty> = res.take(0).unwrap();

    debug!("user bounties {:?}", bounties);

    Json(bounties)
}
