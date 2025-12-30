//! Core Node Engine - Graph-based workflow execution
//!
//! Provides the execution runtime for Houdini-style node graphs.

#[cfg(feature = "backend")]
pub mod circuit_breaker;
pub mod context;
pub mod error;
pub mod events;
pub mod matching;
pub mod nodes;
#[cfg(feature = "backend")]
pub mod notifications;
#[cfg(feature = "backend")]
pub mod payments;
#[cfg(feature = "backend")]
pub mod plugin_sandbox;
pub mod state_machine;
pub mod strategies;
#[cfg(feature = "backend")]
pub mod whatsapp;

// WASM executor and script node only available with backend feature (uses extism/wasmtime)
#[cfg(feature = "backend")]
pub mod wasm_executor;
#[cfg(feature = "backend")]
pub mod script_node;

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

#[cfg(feature = "backend")]
pub use wasm_executor::{WasmExecutor, PluginSource, WasmPluginConfig, HostFunctions};
#[cfg(feature = "backend")]
pub use script_node::ScriptNodeHandler;

// Stub types for WASM frontend builds
#[cfg(not(feature = "backend"))]
pub mod wasm_stubs {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    
    /// Stub for PluginSource when not using backend
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum PluginSource {
        Inline { data: String },
        Url { url: String },
        PluginId { id: Uuid },
    }
    
    /// Stub for WasmPluginConfig when not using backend
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WasmPluginConfig {
        pub source: PluginSource,
        pub function_name: String,
        pub allowed_host_functions: Vec<String>,
        pub memory_limit: Option<usize>,
        pub timeout_ms: Option<u64>,
    }
}

#[cfg(not(feature = "backend"))]
pub use wasm_stubs::{PluginSource, WasmPluginConfig};

