use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use log::debug;
use serde::Deserialize;

use crate::{
    session_auth::{MyAuthSession, MyAuthSessionLayer},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
    // .route("/register", post(register))
    // .route("/login", post(login))
}

/*
#[derive(Debug, Deserialize)]
struct RegisterBody {
    pub username: String,
    pub password: String,
}
async fn register(State(state): State<AppState>, Json(payload): Json<RegisterBody>) {

    // Check that user has not been registered
    let groups = state.db_conn.query("SELECT * FROM Users WHERE username == $username").bind(("username", payload.username)).await.unwrap();
    debug!("groups {groups:?}");

}
*/

/*
#[derive(Debug, Deserialize)]
struct LoginBody {
    pub username: String,
    pub password: String,
}
async fn login(auth: MyAuthSession, State(state): State<AppState>, Json(payload): Json<LoginBody>) {
    // TODO temp hardcoded user id
    auth.login_user(2);
}
*/
