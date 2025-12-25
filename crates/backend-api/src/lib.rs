//! Backend API Library
//! Exports routes and utilities for integration tests

pub mod routes;
pub mod state;
pub mod error;
pub mod config;
pub mod middleware;
pub mod seed;
pub mod events;
pub mod ai;
pub mod cqrs;
pub mod cache;
// pub mod jobs; // TODO: Fix type annotations for never fallback
pub mod gateway;
pub mod websocket;
pub mod performance;
pub mod observability;
