use std::{collections::HashMap, env, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    http::HeaderValue,
    response::Html,
    routing::{get, post},
    Extension, Router,
};
use axum_login::{
    axum_sessions::{async_session::MemoryStore as SessionMemoryStore, SameSite, SessionLayer},
    memory_store::MemoryStore as AuthMemoryStore,
    secrecy::SecretVec,
    AuthLayer, AuthUser, RequireAuthorizationLayer,
};
use axum_server::tls_rustls::RustlsConfig;
use db::DBConnection;
use log::{debug, info, warn};
use rand::Rng;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::Level;

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

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    dotenvy::dotenv().unwrap();

    let app_state = AppState::init().await;

    let secret = rand::thread_rng().gen::<[u8; 64]>();

    let session_store = SessionMemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret)
        .with_secure(true)
        .with_http_only(false)
        .with_same_site_policy(SameSite::None);

    use tokio::sync::RwLock;
    let store: Arc<RwLock<HashMap<String, session_auth::AuthUser>>> =
        Arc::new(RwLock::new(HashMap::default()));

    let dummy_user = session_auth::AuthUser {
        id: String::from("pinosaur"),
    };
    store.write().await.insert(dummy_user.get_id(), dummy_user);

    let user_store = AuthMemoryStore::new(&store);
    let auth_layer = AuthLayer::new(user_store, &secret);

    let cors = CorsLayer::new()
        .allow_origin("http://gitbounties.io:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        // .allow_methods(tower_http::cors::Any)
        .allow_credentials(true);

    let app = Router::new()
        .nest("/", api::router())
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(auth_layer)
        .layer(session_layer);

    // run it with hyper on localhost:3000
    let rustls_config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("key.pem"),
    )
    .await
    .unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
