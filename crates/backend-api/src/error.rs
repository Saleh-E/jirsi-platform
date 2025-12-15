//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use core_auth::AuthError;
use core_metadata::MetadataError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Metadata(#[from] MetadataError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::Auth(e) => match e {
                AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, e.to_string()),
                AuthError::UserNotFound => (StatusCode::NOT_FOUND, e.to_string()),
                AuthError::SessionExpired => (StatusCode::UNAUTHORIZED, e.to_string()),
                _ => (StatusCode::BAD_REQUEST, e.to_string()),
            },
            ApiError::Metadata(e) => (StatusCode::NOT_FOUND, e.to_string()),
            ApiError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
