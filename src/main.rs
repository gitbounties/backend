use std::{collections::HashMap, env, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{
    AuthConfig, AuthSessionLayer, Authentication, SessionNullPool, SessionSqlitePool,
};
use db::DBConnection;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use session_auth::{AuthUser, MyAuthSessionLayer, NullPool};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

mod api;
mod contract;
mod db;
mod ether;
mod models;
mod redis;
mod session_auth;
mod utils;

#[derive(Clone)]
pub struct AppState {
    db_conn: DBConnection,
    /// JWT token used to interact with github REST API
    github_jwt: String,
    /// Reqwest client
    reqwest: reqwest::Client,
}

impl AppState {
    pub async fn init() -> AppState {
        let db_conn = db::connect("127.0.0.1:8000", "admin", "password", "test", "test")
            .await
            .unwrap();

        let reqwest = reqwest::Client::new();
        // TODO this jwt needs to be refreshed every so often
        let github_jwt = utils::generate_github_jwt();
        debug!("github jwt {github_jwt}");
        let app_state = AppState {
            db_conn,
            github_jwt,
            reqwest,
        };

        app_state
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    dotenvy::dotenv().unwrap();

    let app_state = AppState::init().await;

    let pool = SqlitePoolOptions::new()
        .connect("sqlite:auth.db")
        .await
        .expect("Could not make pool.");

    let session_config = SessionConfig::default().with_table_name("axum_sessions");
    let session_store =
        SessionStore::<SessionSqlitePool>::new(Some(pool.clone().into()), session_config)
            .await
            .unwrap();
    let auth_config = AuthConfig::<String>::default();

    // build our application with a single route
    // let nullpool = Arc::new(Option::None);

    let app = Router::new()
        .nest("/", api::router())
        .with_state(app_state)
        .layer(SessionLayer::new(session_store))
        .layer(MyAuthSessionLayer::new(Some(pool)).with_config(auth_config));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
