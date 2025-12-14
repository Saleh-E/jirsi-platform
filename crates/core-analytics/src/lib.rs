//! Core Analytics - Search, Metrics, Dashboards, Targets, and AI
//!
//! Provides full-text search, dashboard metrics, agent targets, and AI-powered features.

pub mod search;
pub mod metrics;
pub mod dashboard;
pub mod targets;

pub use search::*;
pub use metrics::*;
pub use dashboard::*;
pub use targets::*;
