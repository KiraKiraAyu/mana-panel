use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("System error: {0}")]
    System(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            AppError::System(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "SYSTEM_ERROR", msg.clone()),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", e.to_string()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", e.to_string()),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", e.to_string()),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
