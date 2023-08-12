/*
use axum::{
    Router,
    http::{self, Request},
    routing::get,
    response::Response,
    middleware::{self, Next}, extract::{Query, State}, Extension,
};
use reqwest::StatusCode;
use crate::{api::{bounty::IssueQuery, github::get_installation}, AppState, models::User};

async fn with_installation<B>(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    query: Query<IssueQuery>,
    request: Request<B>,
    next: Next<B>,
) -> Response {

    let installation_id =
        if let Some(id) = get_installation(&state, &query.owner, &query.repo).await {
            id
        } else {
            return (StatusCode::NOT_FOUND, "Invalid issue".into());
        };
    let installation_access_token = get_installation_access_token(&state, installation_id).await;

    // Check if user has permission to manage this installation
    let user_data: User = state
        .db_conn
        .select(("Users", &auth_user.id))
        .await
        .expect("User should exist in database");

    if !user_data
        .github_installations
        .contains(&(installation_id as usize))
    {
        return (
            StatusCode::FORBIDDEN,
            "No permission to manage installation".into(),
        );
    }

    let response = next.run(request).await;

    response
}
*/
