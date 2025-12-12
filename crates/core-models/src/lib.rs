//! Core Models - Shared domain types for the SaaS platform
//! 
//! This crate contains all shared structs, enums, and types used across
//! both frontend and backend. It serves as the single source of truth
//! for domain models.

pub mod tenant;
pub mod user;
pub mod entity;
pub mod field;
pub mod association;
pub mod view;
pub mod event;
pub mod node;

// Re-export common types
pub use tenant::*;
pub use user::*;
pub use entity::*;
pub use field::*;
pub use association::*;
pub use view::*;
pub use event::*;
pub use node::*;

