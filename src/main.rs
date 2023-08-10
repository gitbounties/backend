use std::{collections::HashMap, env, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Extension, Router,
};
use axum_login::{
    axum_sessions::{async_session::MemoryStore as SessionMemoryStore, SessionLayer},
    memory_store::MemoryStore as AuthMemoryStore,
    secrecy::SecretVec,
    AuthLayer, RequireAuthorizationLayer,
};
use db::DBConnection;
use log::{debug, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use session_auth::AuthUser;
use tower_http::cors::CorsLayer;

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

    // let pool = SqlitePoolOptions::new()
    //     .connect("sqlite:auth.db")
    //     .await
    //     .expect("Could not make pool.");

    let secret = rand::thread_rng().gen::<[u8; 64]>();

    let session_store = SessionMemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret).with_secure(false);

    use tokio::sync::RwLock;
    let store: Arc<RwLock<HashMap<String, AuthUser>>> = Arc::new(RwLock::new(HashMap::default()));

    let user_store = AuthMemoryStore::new(&store);
    let auth_layer = AuthLayer::new(user_store, &secret);

    let app = Router::new()
        .route("/protected", get(protected))
        .route_layer(RequireAuthorizationLayer::<String, AuthUser>::login())
        .nest("/", api::router())
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(auth_layer)
        .layer(session_layer);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn protected(Extension(user): Extension<AuthUser>) {
    debug!("protected route with {}", user.id);
}
