//! Core Node Engine - Graph-based workflow execution
//!
//! Provides the execution runtime for Houdini-style node graphs.

pub mod context;
pub mod error;
pub mod events;
pub mod nodes;
pub mod strategies;

#[cfg(feature = "backend")]
pub mod executor;
#[cfg(feature = "backend")]
pub mod repository;

pub mod ai;
pub use context::ExecutionContext;
pub use error::NodeEngineError;
pub use events::{EntityEvent, EventType};
#[cfg(feature = "backend")]
pub use events::EventPublisher;

pub use nodes::*; // Re-export node types

#[cfg(feature = "backend")]
pub use executor::GraphExecutor;

pub use strategies::{AssignmentStrategy, AgentStats};
#[cfg(feature = "backend")]
pub use strategies::AssignmentService;

