use axum::{
    routing::{post},
    http::StatusCode,
    response::IntoResponse,
    Router,
};
use tracing_subscriber;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", post(modify_manifest));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn modify_manifest() -> &'static str {
    "Hello, World!"
}
