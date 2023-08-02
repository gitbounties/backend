use axum::{
    routing::{get, post},
    Router,
};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    // build our application with a single route
    let app = Router::new()
        .route("/health", get(health))
        .route("/github/hook", post(github_webhook));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "health!"
}

async fn github_webhook() {
    info!("github hook called!");
}
