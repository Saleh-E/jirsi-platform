//! The Neural Core: Zero UI, Pure Logic.
//! Handles all State, Synchronization, Connectivity, and Data Modeling.

pub mod api;
pub mod context;
pub mod entities;
pub mod models;
pub mod sync_engine;
pub mod shortcuts;

// Re-export core primitives
pub use entities::*;
pub use sync_engine::*;
pub use shortcuts::*;
