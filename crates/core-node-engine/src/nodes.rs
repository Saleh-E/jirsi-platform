//! Node handlers and registry

use async_trait::async_trait;
use core_models::{NodeDef, NodeType};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::executor::ExecutionContext;
use crate::NodeEngineError;

/// Trait for node handlers
#[async_trait]
pub trait NodeHandler: Send + Sync {
    /// Execute the node logic
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError>;
}

/// Registry of node handlers
pub struct NodeRegistry {
    handlers: HashMap<NodeType, Arc<dyn NodeHandler>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };
        
        // Register built-in handlers
        registry.register(NodeType::TriggerOnCreate, Arc::new(TriggerHandler));
        registry.register(NodeType::TriggerOnUpdate, Arc::new(TriggerHandler));
        registry.register(NodeType::TriggerOnDelete, Arc::new(TriggerHandler));
        registry.register(NodeType::DataSetField, Arc::new(SetFieldHandler));
        registry.register(NodeType::ActionSendEmail, Arc::new(SendEmailHandler));
        registry.register(NodeType::ConditionIf, Arc::new(ConditionIfHandler));
        
        registry
    }

    pub fn register(&mut self, node_type: NodeType, handler: Arc<dyn NodeHandler>) {
        self.handlers.insert(node_type, handler);
    }

    pub fn get_handler(&self, node_type: &NodeType) -> Result<Arc<dyn NodeHandler>, NodeEngineError> {
        self.handlers
            .get(node_type)
            .cloned()
            .ok_or_else(|| NodeEngineError::NodeExecutionFailed {
                node_id: uuid::Uuid::nil(),
                message: format!("No handler for node type: {:?}", node_type),
            })
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Built-in Handlers ============

/// Trigger handler - passes through trigger data
pub struct TriggerHandler;

#[async_trait]
impl NodeHandler for TriggerHandler {
    async fn execute(
        &self,
        _node: &NodeDef,
        _inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Return the trigger data
        Ok(context.values.get("$trigger").cloned().unwrap_or(Value::Null))
    }
}

/// Set field handler - modifies a field value
pub struct SetFieldHandler;

#[async_trait]
impl NodeHandler for SetFieldHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let field_name = node.config.get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let value = inputs.get("value").cloned().unwrap_or(Value::Null);
        
        Ok(serde_json::json!({
            "action": "set_field",
            "field": field_name,
            "value": value
        }))
    }
}

/// Send email handler (stub)
pub struct SendEmailHandler;

#[async_trait]
impl NodeHandler for SendEmailHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let to = node.config.get("to")
            .or_else(|| inputs.get("to"))
            .cloned()
            .unwrap_or(Value::Null);
        
        let subject = node.config.get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let _body = node.config.get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // TODO: Actually send email
        tracing::info!(to = ?to, subject = subject, "Would send email");
        
        Ok(serde_json::json!({
            "action": "send_email",
            "to": to,
            "subject": subject,
            "sent": true
        }))
    }
}

/// Condition If handler
pub struct ConditionIfHandler;

#[async_trait]
impl NodeHandler for ConditionIfHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let condition_field = node.config.get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let operator = node.config.get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("equals");
        
        let compare_value = node.config.get("value").cloned().unwrap_or(Value::Null);
        
        let input_value = inputs.get("data")
            .and_then(|v| v.get(condition_field))
            .cloned()
            .unwrap_or(Value::Null);
        
        let result = match operator {
            "equals" => input_value == compare_value,
            "not_equals" => input_value != compare_value,
            "is_null" => input_value.is_null(),
            "is_not_null" => !input_value.is_null(),
            "contains" => {
                input_value.as_str()
                    .map(|s| s.contains(compare_value.as_str().unwrap_or("")))
                    .unwrap_or(false)
            }
            _ => false,
        };
        
        Ok(serde_json::json!({
            "condition": result,
            "field": condition_field,
            "operator": operator
        }))
    }
}

/// Create record handler - creates a new record in an entity
pub struct CreateRecordHandler;

#[async_trait]
impl NodeHandler for CreateRecordHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let entity_type = node.config.get("entity_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Get field values from inputs or config
        let field_values = inputs.get("data")
            .cloned()
            .or_else(|| node.config.get("data").cloned())
            .unwrap_or(Value::Object(serde_json::Map::new()));
        
        // This would be called by the executor with actual DB access
        // For now, return the action metadata
        tracing::info!(
            entity_type = entity_type, 
            data = ?field_values, 
            "Create record action triggered"
        );
        
        Ok(serde_json::json!({
            "action": "create_record",
            "entity_type": entity_type,
            "data": field_values,
            "record_id": uuid::Uuid::new_v4(),
            "created": true
        }))
    }
}

/// Update record handler - updates an existing record
pub struct UpdateRecordHandler;

#[async_trait]
impl NodeHandler for UpdateRecordHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get record ID from trigger data or inputs
        let record_id = inputs.get("record_id")
            .or_else(|| {
                context.values.get("$trigger")
                    .and_then(|t| t.get("record_id"))
            })
            .cloned()
            .unwrap_or(Value::Null);
        
        // Get field updates
        let updates = node.config.get("updates")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));
        
        tracing::info!(
            record_id = ?record_id, 
            updates = ?updates, 
            "Update record action triggered"
        );
        
        Ok(serde_json::json!({
            "action": "update_record",
            "record_id": record_id,
            "updates": updates,
            "updated": true
        }))
    }
}

/// Delete record handler - deletes a record
pub struct DeleteRecordHandler;

#[async_trait]
impl NodeHandler for DeleteRecordHandler {
    async fn execute(
        &self,
        _node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let record_id = inputs.get("record_id")
            .or_else(|| {
                context.values.get("$trigger")
                    .and_then(|t| t.get("record_id"))
            })
            .cloned()
            .unwrap_or(Value::Null);
        
        tracing::info!(record_id = ?record_id, "Delete record action triggered");
        
        Ok(serde_json::json!({
            "action": "delete_record",
            "record_id": record_id,
            "deleted": true
        }))
    }
}
