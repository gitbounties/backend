use std::{collections::HashMap, env, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthSessionLayer, Authentication, SessionNullPool};
use db::DBConnection;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod api;
mod contract;
mod db;
mod ether;
mod models;
mod redis;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthUser {
    pub anonymous: bool,
}

#[axum::async_trait]
impl Authentication<AuthUser, i64, NullPool> for AuthUser {
    // This is ran when the user has logged in and has not yet been Cached in the system.
    // Once ran it will load and cache the user.
    async fn load_user(userid: i64, _pool: Option<&NullPool>) -> anyhow::Result<AuthUser> {
        Ok(AuthUser { anonymous: true })
    }

    // This function is used internally to deturmine if they are logged in or not.
    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}
type NullPool = Arc<Option<()>>;

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    dotenvy::dotenv().unwrap();

    let app_state = AppState::init().await;

    let session_config = SessionConfig::default();
    let session_store = SessionStore::<SessionNullPool>::new(None, session_config)
        .await
        .unwrap();

    // build our application with a single route
    let nullpool = Arc::new(Option::None);

    let app = Router::new()
        .nest("/", api::router())
        .with_state(app_state)
        .layer(SessionLayer::new(session_store))
        .layer(AuthSessionLayer::<AuthUser, i64, SessionNullPool, NullPool>::new(Some(nullpool)));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
