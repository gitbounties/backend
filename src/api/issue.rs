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
    session_auth::AuthUser,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubIssue {
    /// Id for issue
    issue: Issue,
    title: String,
    description: String,
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

    for installation_id in user_data.github_installations.iter() {
        let installation_access_token =
            get_installation_access_token(&state, *installation_id as u64).await;

        // Fetch all issues from all repos in installation
        // TODO can all these calls be parallelized?
        let res = state
            .reqwest_github(
                Method::GET,
                "https://api.github.com/installation/repositories",
                &installation_access_token,
            )
            .send()
            .await
            .unwrap();

        let body = res.json::<serde_json::Value>().await.unwrap();

        for repository in body["repositories"].as_array().unwrap().iter() {
            let repo_owner = repository["owner"]["login"].as_str().unwrap();
            let repo_name = repository["name"].as_str().unwrap();

            let query = format!(
                r#"
              {{
                repository(owner:\"{}\", name:\"{}\") {{
                  issues(states:OPEN) {{
                    edges {{
                      node {{
                        author {{
                          login
                        }}
                        title
                        body
                        labels(first: 10) {{
                          edges {{
                            node {{
                              name
                            }}
                          }}
                        }}
                      }}
                    }}
                  }}
                }}
              }}
            "#,
                repo_owner, repo_name
            )
            .replace("\\n", "");

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

            debug!("got issue {:?}", body);
        }
    }

    Json(vec![])
}
