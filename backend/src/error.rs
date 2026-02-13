use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerErrorKind {
    DaemonUnavailable,
    NotFound,
    ImageNotFound,
    ContainerNotFound,
    Conflict,
    PortConflict,
    NameConflict,
    PermissionDenied,
    RegistryAuth,
    Timeout,
    InvalidReference,
    Unknown,
}

impl fmt::Display for DockerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DockerErrorKind::DaemonUnavailable => "daemon_unavailable",
            DockerErrorKind::NotFound => "not_found",
            DockerErrorKind::ImageNotFound => "image_not_found",
            DockerErrorKind::ContainerNotFound => "container_not_found",
            DockerErrorKind::Conflict => "conflict",
            DockerErrorKind::PortConflict => "port_conflict",
            DockerErrorKind::NameConflict => "name_conflict",
            DockerErrorKind::PermissionDenied => "permission_denied",
            DockerErrorKind::RegistryAuth => "registry_auth",
            DockerErrorKind::Timeout => "timeout",
            DockerErrorKind::InvalidReference => "invalid_reference",
            DockerErrorKind::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

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

    #[error("Docker error [{operation}] ({kind}): {message}")]
    Docker {
        operation: String,
        kind: DockerErrorKind,
        message: String,
    },

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn docker(
        operation: impl Into<String>,
        kind: DockerErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self::Docker {
            operation: operation.into(),
            kind,
            message: message.into(),
        }
    }

    pub fn is_docker_image_not_found(&self) -> bool {
        matches!(
            self,
            AppError::Docker {
                kind: DockerErrorKind::ImageNotFound,
                ..
            }
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = match &self {
            AppError::Auth(msg) => Json(json!({
                "success": false,
                "error": {
                    "code": "AUTH_ERROR",
                    "message": msg
                }
            })),
            AppError::Validation(msg) => Json(json!({
                "success": false,
                "error": {
                    "code": "VALIDATION_ERROR",
                    "message": msg
                }
            })),
            AppError::NotFound(msg) => Json(json!({
                "success": false,
                "error": {
                    "code": "NOT_FOUND",
                    "message": msg
                }
            })),
            AppError::Forbidden(msg) => Json(json!({
                "success": false,
                "error": {
                    "code": "FORBIDDEN",
                    "message": msg
                }
            })),
            AppError::System(msg) => Json(json!({
                "success": false,
                "error": {
                    "code": "SYSTEM_ERROR",
                    "message": msg
                }
            })),
            AppError::Docker {
                operation,
                kind,
                message,
            } => Json(json!({
                "success": false,
                "error": {
                    "code": "DOCKER_ERROR",
                    "message": message,
                    "docker": {
                        "operation": operation,
                        "kind": kind.to_string()
                    }
                }
            })),
            AppError::Database(e) => Json(json!({
                "success": false,
                "error": {
                    "code": "DATABASE_ERROR",
                    "message": e.to_string()
                }
            })),
            AppError::Io(e) => Json(json!({
                "success": false,
                "error": {
                    "code": "IO_ERROR",
                    "message": e.to_string()
                }
            })),
            AppError::Internal(e) => Json(json!({
                "success": false,
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        };

        let status = match &self {
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::System(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Docker { kind, .. } => match kind {
                DockerErrorKind::DaemonUnavailable => StatusCode::SERVICE_UNAVAILABLE,
                DockerErrorKind::ImageNotFound | DockerErrorKind::ContainerNotFound => {
                    StatusCode::NOT_FOUND
                }
                DockerErrorKind::Conflict
                | DockerErrorKind::PortConflict
                | DockerErrorKind::NameConflict => StatusCode::CONFLICT,
                DockerErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
                DockerErrorKind::RegistryAuth => StatusCode::UNAUTHORIZED,
                DockerErrorKind::InvalidReference => StatusCode::BAD_REQUEST,
                DockerErrorKind::Timeout => StatusCode::GATEWAY_TIMEOUT,
                DockerErrorKind::NotFound => StatusCode::NOT_FOUND,
                DockerErrorKind::Unknown => StatusCode::BAD_GATEWAY,
            },
            AppError::Database(_) | AppError::Io(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
