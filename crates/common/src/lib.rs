use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("external service error: {0}")]
    External(String),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub pennylane_base_url: String,
    pub pennylane_api_token: Option<String>,
    pub sendcloud_base_url: String,
    pub sendcloud_public_key: Option<String>,
    pub sendcloud_private_key: Option<String>,
    pub sendcloud_integration_id: Option<i64>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            pennylane_base_url: std::env::var("PENNYLANE_BASE_URL")
                .unwrap_or_else(|_| "https://app.pennylane.com".to_string()),
            pennylane_api_token: std::env::var("PENNYLANE_API_TOKEN").ok(),
            sendcloud_base_url: std::env::var("SENDCLOUD_BASE_URL")
                .unwrap_or_else(|_| "https://panel.sendcloud.sc/api/v3".to_string()),
            sendcloud_public_key: std::env::var("SENDCLOUD_PUBLIC_KEY").ok(),
            sendcloud_private_key: std::env::var("SENDCLOUD_PRIVATE_KEY").ok(),
            sendcloud_integration_id: std::env::var("SENDCLOUD_INTEGRATION_ID")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }
}
