
use serde_json::Value;
use std::collections::HashMap;

use std::sync::Arc;
use crate::ai::AiService;

/// Execution context - holds values during graph execution
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub values: HashMap<String, Value>,
    pub logs: Vec<Value>,
    pub ai_service: Option<Arc<dyn AiService>>,
    /// Data from the trigger that started this execution
    pub trigger_data: Value,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_ai_service(mut self, service: Arc<dyn AiService>) -> Self {
        self.ai_service = Some(service);
        self
    }
    
    pub fn with_trigger_data(mut self, data: Value) -> Self {
        self.trigger_data = data;
        self
    }
}

