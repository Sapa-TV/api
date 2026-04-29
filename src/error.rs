use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Environment variable error: {0}")]
    Env(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    #[allow(dead_code)]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Migration(e) => {
                tracing::error!("Migration error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Env(msg) => {
                tracing::error!("Environment error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Unauthorized(msg) => {
                tracing::warn!("Unauthorized: {}", msg);
                (StatusCode::OK, self.to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(ErrorResponse { error: message });

        (status, body).into_response()
    }
}
