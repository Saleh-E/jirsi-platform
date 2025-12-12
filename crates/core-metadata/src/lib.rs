//! Core Metadata Engine
//! 
//! Provides access to EntityTypes, FieldDefs, ViewDefs, and other metadata.
//! Handles caching and tenant isolation for metadata queries.

pub mod repository;
pub mod cache;
pub mod service;
pub mod error;

pub use error::MetadataError;
pub use service::MetadataService;
