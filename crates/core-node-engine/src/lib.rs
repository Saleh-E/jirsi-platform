//! Core Node Engine - Graph-based workflow execution
//!
//! Provides the execution runtime for Houdini-style node graphs.

pub mod error;
pub mod events;
pub mod executor;
pub mod nodes;
pub mod repository;

pub use error::NodeEngineError;
pub use events::{EntityEvent, EventPublisher, EventType};
pub use executor::GraphExecutor;
