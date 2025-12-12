//! Core Auth - Authentication and authorization services
//!
//! Provides tenant management, user authentication, sessions, and role-based access control.

pub mod error;
pub mod password;
pub mod session;
pub mod tenant;
pub mod user;
pub mod middleware;

pub use error::AuthError;
