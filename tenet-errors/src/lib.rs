mod conversions;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    ValidationError(String),
    #[error("request validation failed")]
    Invalid(Vec<FieldError>),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    InternalError(String),
}

impl AppError {
    pub fn internal(err: impl std::fmt::Display) -> Self {
        AppError::InternalError(err.to_string())
    }

    pub fn bad_request(err: impl std::fmt::Display) -> Self {
        AppError::BadRequest(err.to_string())
    }

    pub fn unavailable(err: impl std::fmt::Display) -> Self {
        AppError::Unavailable(err.to_string())
    }

    pub fn not_found(err: impl std::fmt::Display) -> Self {
        AppError::NotFound(err.to_string())
    }

    pub fn is_transient(&self) -> bool {
        matches!(self, AppError::Unavailable(_))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FieldError>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, plain(msg)),
            AppError::BadRequest(msg) | AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, plain(msg))
            }
            AppError::Invalid(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse {
                    message: "request validation failed".to_owned(),
                    errors: Some(fields),
                },
            ),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, plain(msg)),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, plain(msg)),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, plain(msg)),
            AppError::Unavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, plain(msg)),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, plain(msg)),
        };
        (status, Json(body)).into_response()
    }
}

fn plain(message: String) -> ErrorResponse {
    ErrorResponse {
        message,
        errors: None,
    }
}
