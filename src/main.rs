use std::{collections::HashMap, env};

use axum::{
    extract::{Json, Path, Query},
    response::Html,
    routing::{get, post},
    Router,
};
use log::{debug, info, warn};
use models::Issue;
use serde_json::json;

mod api;
mod db;
mod models;

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    dotenvy::dotenv().unwrap();

    db::connect("127.0.0.1:8000").await.unwrap();

    // build our application with a single route
    let app = api::router();

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
