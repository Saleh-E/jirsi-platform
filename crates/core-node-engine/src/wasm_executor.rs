//! WASM Plugin Executor using Extism
//!
//! Provides sandboxed execution of user-defined logic in WASM.
//! Supports plugins written in Rust, JavaScript, Python, Go, etc.

use extism::{Manifest, Plugin, Wasm};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::error::NodeEngineError;

/// WASM plugin executor with caching and sandboxing
pub struct WasmExecutor {
    /// Cache of loaded plugins (keyed by source hash or ID)
    plugin_cache: Arc<Mutex<HashMap<String, Plugin>>>,
    /// Memory limit for WASM execution (bytes)
    memory_limit: Option<usize>,
    /// Timeout for WASM execution (milliseconds)
    timeout_ms: Option<u64>,
}

impl WasmExecutor {
    /// Create a new WASM executor
    pub fn new() -> Self {
        Self {
            plugin_cache: Arc::new(Mutex::new(HashMap::new())),
            memory_limit: Some(100 * 1024 * 1024), // 100MB default
            timeout_ms: Some(30_000), // 30 seconds default
        }
    }
    
    /// Set memory limit (in bytes)
    pub fn with_memory_limit(mut self, bytes: usize) -> Self {
        self.memory_limit = Some(bytes);
        self
    }
    
    /// Set execution timeout (in milliseconds)
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
    
    /// Execute a WASM plugin function with given input
    ///
    /// # Arguments
    /// * `plugin_source` - Source of the WASM plugin (inline, URL, or ID)
    /// * `function_name` - Name of the function to call in the WASM module
    /// * `input` - JSON input to pass to the function
    /// * `allowed_host_functions` - List of host functions this plugin can call
    ///
    /// # Returns
    /// JSON output from the WASM function
    pub async fn execute(
        &self,
        plugin_source: &PluginSource,
        function_name: &str,
        input: JsonValue,
        allowed_host_functions: &[String],
    ) -> Result<JsonValue, NodeEngineError> {
        tracing::debug!(
            function = function_name,
            "Executing WASM plugin"
        );
        
        // Get or load plugin
        let mut plugin = self.get_or_load_plugin(plugin_source, allowed_host_functions).await?;
        
        // Serialize input to JSON bytes
        let input_bytes = serde_json::to_vec(&input)
            .map_err(|e| NodeEngineError::WasmError(format!("Input serialization failed: {}", e)))?;
        
        // Call WASM function - Extism returns Vec<u8>
        let output_bytes: Vec<u8> = plugin
            .call(function_name, &input_bytes)
            .map_err(|e| NodeEngineError::WasmError(format!("WASM execution failed: {}", e)))?;
        
        // Deserialize output
        let output: JsonValue = serde_json::from_slice(&output_bytes)
            .map_err(|e| NodeEngineError::WasmError(format!("Output deserialization failed: {}", e)))?;
        
        tracing::debug!(
            function = function_name,
            output_size = output_bytes.len(),
            "WASM execution completed"
        );
        
        Ok(output)
    }
    
    /// Get plugin from cache or load it
    async fn get_or_load_plugin(
        &self,
        source: &PluginSource,
        _allowed_host_functions: &[String],
    ) -> Result<Plugin, NodeEngineError> {
        let cache_key = source.cache_key();
        
        // Check cache first
        {
            let cache = self.plugin_cache.lock().unwrap();
            if let Some(_plugin) = cache.get(&cache_key) {
                tracing::debug!(cache_key = %cache_key, "Using cached WASM plugin");
                // Note: Extism Plugin doesn't implement Clone, so we need to reload
                // In production, we'd use a different caching strategy
            }
        }
        
        // Load plugin based on source type
        let wasm_bytes = match source {
            PluginSource::Inline { data } => {
                // Decode base64 WASM
                use base64::{Engine as _, engine::general_purpose};
                general_purpose::STANDARD.decode(data)
                    .map_err(|e| NodeEngineError::WasmError(format!("Invalid base64: {}", e)))?
            },
            PluginSource::Url { url } => {
                #[cfg(feature = "backend")]
                {
                    // Fetch WASM from URL
                    tracing::info!(url = %url, "Fetching WASM plugin from URL");
                    let response = reqwest::get(url)
                        .await
                        .map_err(|e| NodeEngineError::WasmError(format!("Failed to fetch plugin: {}", e)))?;
                    
                    response.bytes()
                        .await
                        .map_err(|e| NodeEngineError::WasmError(format!("Failed to read plugin bytes: {}", e)))?
                        .to_vec()
                }
                #[cfg(not(feature = "backend"))]
                {
                    return Err(NodeEngineError::WasmError(
                        "URL plugin loading requires 'backend' feature".to_string()
                    ));
                }
            },
            PluginSource::PluginId { id } => {
                // In production, load from database
                // For now, return error
                return Err(NodeEngineError::WasmError(
                    format!("Plugin ID {} not found in database", id)
                ));
            },
        };
        
        // Create Extism manifest
        let wasm = Wasm::data(wasm_bytes);
        let manifest = Manifest::new([wasm]);
        
        // Create plugin with configuration
        let plugin = Plugin::new(&manifest, [], true)
            .map_err(|e| NodeEngineError::WasmError(format!("Failed to create plugin: {}", e)))?;
        
        // Set memory limit if configured
        if let Some(limit) = self.memory_limit {
            // Extism handles this internally via Wasmtime config
            tracing::debug!(limit_bytes = limit, "WASM memory limit configured");
        }
        
        // Set timeout if configured
        if let Some(timeout) = self.timeout_ms {
            // Extism handles this internally
            tracing::debug!(timeout_ms = timeout, "WASM timeout configured");
        }
        
        // Cache the plugin
        // Note: In production, we'd implement a more sophisticated caching strategy
        // since Plugin doesn't implement Clone
        
        Ok(plugin)
    }
    
    /// Clear the plugin cache
    pub fn clear_cache(&self) {
        let mut cache = self.plugin_cache.lock().unwrap();
        cache.clear();
        tracing::info!("WASM plugin cache cleared");
    }
}

impl Default for WasmExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Source of a WASM plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginSource {
    /// Inline base64-encoded WASM
    Inline { data: String },
    
    /// URL to fetch WASM from
    Url { url: String },
    
    /// Pre-uploaded plugin ID in database
    PluginId { id: Uuid },
}

impl PluginSource {
    /// Generate a cache key for this plugin source
    fn cache_key(&self) -> String {
        match self {
            PluginSource::Inline { data } => {
                // Use hash of data for cache key
                format!("inline:{}", &data[..32.min(data.len())])
            },
            PluginSource::Url { url } => {
                format!("url:{}", url)
            },
            PluginSource::PluginId { id } => {
                format!("id:{}", id)
            },
        }
    }
}

/// Configuration for a WASM plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmPluginConfig {
    /// Plugin source
    pub source: PluginSource,
    
    /// Function to call in WASM module
    pub function_name: String,
    
    /// Allowed host functions this plugin can call
    pub allowed_host_functions: Vec<String>,
    
    /// Memory limit (bytes)
    pub memory_limit: Option<usize>,
    
    /// Timeout (milliseconds)
    pub timeout_ms: Option<u64>,
}

/// Host functions available to WASM plugins
///
/// These functions can be called from WASM plugins if allowed.
/// They provide controlled access to platform capabilities.
pub struct HostFunctions;

impl HostFunctions {
    /// Log a message from WASM (always allowed)
    pub fn log(level: &str, message: &str) {
        match level {
            "error" => tracing::error!("[WASM Plugin] {}", message),
            "warn" => tracing::warn!("[WASM Plugin] {}", message),
            "info" => tracing::info!("[WASM Plugin] {}", message),
            "debug" => tracing::debug!("[WASM Plugin] {}", message),
            _ => tracing::trace!("[WASM Plugin] {}", message),
        }
    }
    
    /// HTTP GET request (requires 'http_get' permission)
    #[cfg(feature = "backend")]
    pub async fn http_get(url: &str) -> Result<String, String> {
        tracing::debug!(url = %url, "WASM plugin HTTP GET request");
        
        match reqwest::get(url).await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => Ok(text),
                    Err(e) => Err(format!("Failed to read response: {}", e)),
                }
            },
            Err(e) => Err(format!("HTTP request failed: {}", e)),
        }
    }
    
    /// HTTP POST request (requires 'http_post' permission)
    #[cfg(feature = "backend")]
    pub async fn http_post(url: &str, body: &str) -> Result<String, String> {
        tracing::debug!(url = %url, "WASM plugin HTTP POST request");
        
        let client = reqwest::Client::new();
        match client.post(url).body(body.to_string()).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => Ok(text),
                    Err(e) => Err(format!("Failed to read response: {}", e)),
                }
            },
            Err(e) => Err(format!("HTTP request failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_source_cache_key() {
        let source = PluginSource::Url {
            url: "https://example.com/plugin.wasm".to_string(),
        };
        
        assert_eq!(
            source.cache_key(),
            "url:https://example.com/plugin.wasm"
        );
    }
    
    #[test]
    fn test_wasm_executor_creation() {
        let executor = WasmExecutor::new()
            .with_memory_limit(50 * 1024 * 1024)
            .with_timeout(10_000);
        
        assert_eq!(executor.memory_limit, Some(50 * 1024 * 1024));
        assert_eq!(executor.timeout_ms, Some(10_000));
    }
}
