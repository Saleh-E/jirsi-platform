//! CQRS Infrastructure
//! 
//! Command-Query Responsibility Segregation (CQRS) implementation
//! using the esrs (Event Sourcing RS) crate.

pub mod commands;
pub mod events;
pub mod aggregates;
pub mod projections;
pub mod event_store;

#[cfg(test)]
mod tests;

pub use commands::*;
pub use events::*;
pub use aggregates::*;
pub use event_store::*;
