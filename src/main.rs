use std::{collections::HashMap, env, sync::Arc};

use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use db::DBConnection;
use log::{debug, info, warn};
use models::Issue;
use serde_json::json;

mod api;
mod contract;
mod db;
mod ether;
mod models;

#[derive(Clone)]
pub struct AppState {
    db_conn: DBConnection,
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    dotenvy::dotenv().unwrap();

    let db_conn = db::connect("127.0.0.1:8000", "admin", "password", "test", "test")
        .await
        .unwrap();

    let app_state = AppState { db_conn };

    // build our application with a single route
    let app = Router::new().nest("/", api::router()).with_state(app_state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
