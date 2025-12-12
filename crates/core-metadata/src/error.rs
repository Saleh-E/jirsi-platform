//! Metadata errors

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Entity type not found: {0}")]
    EntityTypeNotFound(String),

    #[error("Entity type ID not found: {0}")]
    EntityTypeIdNotFound(Uuid),

    #[error("Field not found: {entity}.{field}")]
    FieldNotFound { entity: String, field: String },

    #[error("View not found: {0}")]
    ViewNotFound(String),

    #[error("Association not found: {0}")]
    AssociationNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
