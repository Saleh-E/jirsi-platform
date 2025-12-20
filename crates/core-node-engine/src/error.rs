//! Node engine errors

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum NodeEngineError {
    #[error("Graph not found: {0}")]
    GraphNotFound(Uuid),

    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Cycle detected in graph")]
    CycleDetected,

    #[error("Node execution failed: {node_id} - {message}")]
    NodeExecutionFailed { node_id: Uuid, message: String },

    #[error("Invalid port connection: {0}")]
    InvalidPortConnection(String),

    #[error("Missing required input: {node_id}.{port}")]
    MissingInput { node_id: Uuid, port: String },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Execution timeout")]
    Timeout,

    #[error("Max retries exceeded")]
    MaxRetriesExceeded,

    #[cfg(feature = "backend")]
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
