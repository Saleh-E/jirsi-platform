//! Core Models - System DNA for the Jirsi Platform
//! 
//! This crate contains all shared structs, enums, and types used across
//! both frontend and backend. It serves as the single source of truth
//! for domain models.
//!
//! ## Antigravity Diamond Protocol
//! 
//! The core-models crate implements the Diamond Protocol, where each
//! field definition has four dimensions:
//! - **Data**: Field types (Text, Number, Money, etc.)
//! - **Logic**: Conditional visibility and editability rules
//! - **Physics**: CRDT sync strategies for conflict resolution
//! - **Intelligence**: AI/LLM metadata for smart features

// ============================================================================
// ANTIGRAVITY MODULES
// ============================================================================

/// Logic Engine - Conditional evaluation for visibility/editability
pub mod logic;

/// Diamond Metadata - Physics (sync) and Intelligence (AI) layers
pub mod metadata;

/// Hybrid Validation - Portable (WASM) + Async (DB) validators
pub mod validation;

// ============================================================================
// DOMAIN MODULES
// ============================================================================

pub mod tenant;
pub mod user;
pub mod entity;
pub mod field;
pub mod association;
pub mod view;
pub mod node;
pub mod crdt;
pub mod sync;
pub mod event;

// ============================================================================
// RE-EXPORTS
// ============================================================================

// Antigravity layers
pub use logic::*;
pub use metadata::*;
pub use validation::*;

// Domain types
pub use tenant::*;
pub use user::*;
pub use entity::*;
pub use field::*;
pub use association::*;
pub use view::*;
pub use event::*;
pub use node::*;
