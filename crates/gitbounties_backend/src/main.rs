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
use clap::Parser;
use db::DBConnection;
use log::{debug, info, warn};
use rand::Rng;
use reqwest::{header, IntoUrl, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod api;
mod contract;
mod db;
mod ether;
mod middleware;
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
        let db_conn = db::connect(
            &env::var("DB_URL").expect("Couldn't get DB_URL env var"),
            &env::var("DB_USERNAME").expect("Couldn't get DB_USERNAME env var"),
            &env::var("DB_PASSWORD").expect("Couldn't get DB_PASSWORD env var"),
            &env::var("DB_NAMESPACE").expect("Couldn't get DB_NAMESPACE env var"),
            &env::var("DB_DATABASE").expect("Couldn't get DB_DATABASE env var"),
        )
        .await
        .unwrap();

        // db::migrate(&db_conn).await;
        // db::migrate_dummy(&db_conn).await;

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

    pub fn reqwest_github<U: IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
        auth: &str,
    ) -> RequestBuilder {
        self.reqwest
            .request(method, url)
            .header("User-Agent", "GitBounties")
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(auth)
    }
}

#[derive(Parser, Debug)]
#[command(name = "gitbounties")]
#[command(bin_name = "gitbounties")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Flag to disable HTTPS
    #[arg(long)]
    no_https: bool,
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let cli = Cli::parse();

    if dotenvy::dotenv().is_err() {
        warn!("Error reading .env file");
    } else {
        debug!("Loaded env vars =-=-=-=");
        for item in dotenvy::dotenv_iter().unwrap() {
            let (key, val) = item.unwrap();
            debug!("{}={}", key, val);
        }
    }

    //debug!("secret {}", env::var("CLIENT_PRIVATE_KEY").unwrap());

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

    // TODO not sure why need to do this
    let dummy_user = session_auth::AuthUser {
        id: String::from("MrPicklePinosaur"),
    };
    store.write().await.insert(dummy_user.get_id(), dummy_user);
    let dummy_user = session_auth::AuthUser {
        id: String::from("MrPicklePinosaur2"),
    };
    store.write().await.insert(dummy_user.get_id(), dummy_user);

    let user_store = AuthMemoryStore::new(&store);
    let auth_layer = AuthLayer::new(user_store, &secret);

    let origins = [
        "http://gitbounties.io:3000".parse().unwrap(),
        "https://gitbounties.io:3000".parse().unwrap(),
        "http://localhost:3000".parse().unwrap(),
        "https://gitbounties.karatsubalabs.com".parse().unwrap(),
    ];
    let cors = CorsLayer::new()
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .allow_origin(origins)
        // .allow_methods(tower_http::cors::Any)
        .allow_credentials(true);

    let app = Router::new()
        .nest("/", api::router())
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(auth_layer)
        .layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    if cli.no_https {
        info!("Starting server with HTTPS disabled...");

        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        info!("Starting server with HTTPS...");

        // run it with hyper on localhost:3000
        // TODO not very nice how we need to reach out of the crates/ dir to get to workspace root
        let rustls_config = RustlsConfig::from_pem_file(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("certs")
                .join("cert.pem"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("certs")
                .join("key.pem"),
        )
        .await
        .unwrap();

        axum_server::bind_rustls(addr, rustls_config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
