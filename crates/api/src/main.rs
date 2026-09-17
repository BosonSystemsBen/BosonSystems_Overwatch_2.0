mod error;
mod routes;
mod state;

use axum::{routing::get, Router};
use common::Config;
use sea_orm::Database;
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    let db = Database::connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(routes::products::router())
        .merge(routes::serial_numbers::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("failed to bind listener");

    tracing::info!("listening on {}", config.listen_addr);
    axum::serve(listener, app).await.expect("server error");
}
