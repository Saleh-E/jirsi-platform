//! ScriptNode handler - executes user-defined WASM logic
//!
//! This node allows users to write custom logic in any language that
//! compiles to WASM (Rust, JavaScript, Python, Go, etc).

use async_trait::async_trait;
use core_models::NodeDef;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::context::ExecutionContext;
use crate::nodes::NodeHandler;
use crate::wasm_executor::{PluginSource, WasmExecutor};
use crate::NodeEngineError;

/// ScriptNode handler - executes WASM plugins
pub struct ScriptNodeHandler {
    /// WASM executor instance
    executor: WasmExecutor,
}

impl ScriptNodeHandler {
    pub fn new() -> Self {
        Self {
            executor: WasmExecutor::new()
                .with_memory_limit(100 * 1024 * 1024) // 100MB
                .with_timeout(30_000), // 30 seconds
        }
    }
    
    /// Create with custom executor configuration
    pub fn with_executor(executor: WasmExecutor) -> Self {
        Self { executor }
    }
}

impl Default for ScriptNodeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeHandler for ScriptNodeHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        tracing::info!(node_id = %node.id, "Executing ScriptNode");
        
        // Extract plugin configuration from node config
        let config = &node.config;
        
        // Get plugin source  
        let source_type = config.get("source_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::NodeExecutionFailed {
                node_id: node.id,
                message: "Missing source_type in ScriptNode config".to_string(),
            })?;
        
        let plugin_source = match source_type {
            "inline" => {
                let data = config.get("wasm_base64")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: "Missing wasm_base64 for inline plugin".to_string(),
                    })?;
                
                PluginSource::Inline {
                    data: data.to_string(),
                }
            },
            "url" => {
                let url = config.get("wasm_url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: "Missing wasm_url for URL plugin".to_string(),
                    })?;
                
                PluginSource::Url {
                    url: url.to_string(),
                }
            },
            "plugin_id" => {
                let id_str = config.get("plugin_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: "Missing plugin_id".to_string(),
                    })?;
                
                let id = uuid::Uuid::parse_str(id_str)
                    .map_err(|e| NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: format!("Invalid plugin_id: {}", e),
                    })?;
                
                PluginSource::PluginId { id }
            },
            _ => {
                return Err(NodeEngineError::NodeExecutionFailed {
                    node_id: node.id,
                    message: format!("Unknown source_type: {}", source_type),
                });
            }
        };
        
        // Get function name
        let function_name = config.get("function_name")
            .and_then(|v| v.as_str())
            .unwrap_or("execute"); // Default function name
        
        // Get allowed host functions
        let allowed_host_functions = config.get("allowed_host_functions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["log".to_string()]); // Default: only logging
        
        // Prepare input for WASM
        let wasm_input = json!({
            "inputs": inputs,
            "node_id": node.id.to_string(),
            "node_label": node.label,
        });
        
        // Execute WASM plugin
        let output = self.executor
            .execute(&plugin_source, function_name, wasm_input, &allowed_host_functions)
            .await
            .map_err(|e| NodeEngineError::NodeExecutionFailed {
                node_id: node.id,
                message: format!("WASM execution failed: {}", e),
            })?;
        
        tracing::info!(
            node_id = %node.id,
            function = function_name,
            "ScriptNode execution completed"
        );
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_script_node_missing_config() {
        let handler = ScriptNodeHandler::new();
        
        let node = NodeDef {
            id: Uuid::new_v4(),
            graph_id: Uuid::new_v4(),
            node_type: core_models::NodeType::ScriptNode,
            label: "Test Script".to_string(),
            x: 0.0,
            y: 0.0,
            config: json!({}), // Empty config
            is_enabled: true,
        };
        
        let inputs = HashMap::new();
        let mut context = ExecutionContext {
            values: HashMap::new(),
            logs: Vec::new(),
            ai_service: None,
        };
        
        let result = handler.execute(&node, inputs, &mut context).await;
        
        assert!(result.is_err());
        match result {
            Err(NodeEngineError::NodeExecutionFailed { message, .. }) => {
                assert!(message.contains("source_type"));
            },
            _ => panic!("Expected NodeExecutionFailed error"),
        }
    }
}
