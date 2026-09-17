use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        }
    }
}
