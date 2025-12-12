//! Auth errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists: {0}")]
    UserExists(String),

    #[error("Tenant not found")]
    TenantNotFound,

    #[error("Tenant not active")]
    TenantNotActive,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Account disabled")]
    AccountDisabled,

    #[error("Invalid invitation")]
    InvalidInvitation,

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Password hash error")]
    PasswordHash,
}
