use axum::{http::StatusCode, response::IntoResponse, Json};
use common::{AppError, ErrorBody};

pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.0.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
            AppError::External(_) => (StatusCode::BAD_GATEWAY, self.0.to_string()),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}
