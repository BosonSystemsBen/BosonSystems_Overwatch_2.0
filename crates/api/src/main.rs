mod error;
mod routes;
mod state;

use axum::{response::Html, routing::get, Router};
use common::Config;
use pennylane::PennylaneClient;
use sea_orm::Database;
use sendcloud::SendcloudClient;
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    let db = Database::connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    let pennylane = config
        .pennylane_api_token
        .clone()
        .map(|token| PennylaneClient::new(config.pennylane_base_url.clone(), token));

    let sendcloud = match (
        config.sendcloud_public_key.clone(),
        config.sendcloud_private_key.clone(),
        config.sendcloud_integration_id,
    ) {
        (Some(public_key), Some(private_key), Some(integration_id)) => Some((
            SendcloudClient::new(config.sendcloud_base_url.clone(), public_key, private_key),
            integration_id,
        )),
        _ => None,
    };

    let state = AppState { db, pennylane, sendcloud };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/",
            get(|| async { Html(include_str!("../static/index.html")) }),
        )
        .merge(routes::products::router())
        .merge(routes::serial_numbers::router())
        .merge(routes::shipments::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("failed to bind listener");

    tracing::info!("listening on {}", config.listen_addr);
    axum::serve(listener, app).await.expect("server error");
}
