use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::github::get_installation_access_token;
use crate::{
    models::{Issue, User},
    session_auth::{AuthUser, MyRequireAuthorizationLayer},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list).layer(MyRequireAuthorizationLayer::login()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubIssue {
    /// Id for issue
    issue: Issue,
    title: String,
    description: String,
    /// Who created the issue
    author: String,
}

/// Return issues from github
pub async fn list(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Json<Vec<GithubIssue>> {
    // Get all installations
    let user_data: User = state
        .db_conn
        .select(("Users", &auth_user.id))
        .await
        .expect("User should exist in database");

    let mut issues: Vec<GithubIssue> = vec![];

    for installation_id in user_data.github_installations.iter() {
        let installation_access_token =
            get_installation_access_token(&state, *installation_id as u64).await;

        // Fetch all issues from all repos in installation
        // TODO can all these calls be parallelized?
        // TODO limited to only 100 repos (maybe imeplement pagination?)
        let res = state
            .reqwest_github(
                Method::GET,
                "https://api.github.com/installation/repositories?per_page=100",
                &installation_access_token,
            )
            .send()
            .await
            .unwrap();

        let body = res.json::<serde_json::Value>().await.unwrap();

        let repositories = body["repositories"].as_array().unwrap();

        for repository in repositories.iter() {
            let repo_owner = repository["owner"]["login"].as_str().unwrap();
            let repo_name = repository["name"].as_str().unwrap();

            //debug!("{repo_owner}/{repo_name}");

            let query = format!(
                r#" {{ repository(owner:\"{}\", name:\"{}\") {{ issues(last:100, states:OPEN) {{ edges {{ node {{ author {{ login }} title body number labels(first: 10) {{ edges {{ node {{ name }} }} }} }} }} }} }} }} "#,
                repo_owner, repo_name
            );

            let res = state
                .reqwest_github(
                    Method::POST,
                    "https://api.github.com/graphql",
                    &installation_access_token,
                )
                .body(format!(r#"{{ "query": "{query}" }}"#))
                .send()
                .await
                .unwrap();

            let body = res.json::<serde_json::Value>().await.unwrap();

            //debug!("got issue {:?}", body);

            let issues_raw = body["data"]["repository"]["issues"]["edges"]
                .as_array()
                .unwrap();

            for issue_raw in issues_raw.iter() {
                let issue_raw = &issue_raw["node"];

                let issue_id = issue_raw["number"].as_u64().unwrap();
                let author = issue_raw["author"]["login"].as_str().unwrap();
                let description = issue_raw["body"].as_str().unwrap();
                let title = issue_raw["title"].as_str().unwrap();

                issues.push(GithubIssue {
                    issue: Issue {
                        owner: repo_owner.into(),
                        repo: repo_name.into(),
                        issue_id: issue_id as usize,
                    },
                    title: title.into(),
                    description: description.into(),
                    author: author.into(),
                });
            }
        }
    }

    Json(issues)
}
