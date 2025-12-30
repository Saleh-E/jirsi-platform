//! Node handlers and registry

use async_trait::async_trait;
use core_models::{NodeDef, NodeType};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::context::ExecutionContext;
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
        registry.register(NodeType::AiGenerate, Arc::new(AiGenerateHandler));
        
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
    let mut registry = Self::new();
    
    // Register built-in handlers
    registry.register(
        NodeType::TriggerManual,
        Arc::new(TriggerHandler),
    );
    registry.register(
        NodeType::DataSetField,
        Arc::new(SetFieldHandler),
    );
    registry.register(
        NodeType::ActionSendEmail,
        Arc::new(SendEmailHandler),
    );
    registry.register(
        NodeType::ConditionIf,
        Arc::new(ConditionIfHandler),
    );
    registry.register(
        NodeType::DataCreateRecord,
        Arc::new(CreateRecordHandler),
    );
    registry.register(
        NodeType::DataUpdateRecord,
        Arc::new(UpdateRecordHandler),
    );
    
    // External Integration Nodes
    registry.register(
        NodeType::ActionSendSms,
        Arc::new(SendSmsHandler),
    );
    registry.register(
        NodeType::ActionSendWebhook,
        Arc::new(WebhookHandler),
    );
    registry.register(
        NodeType::ActionDelay,
        Arc::new(DelayHandler),
    );
    
    // AI-Powered Nodes
    registry.register(
        NodeType::AiGenerate,
        Arc::new(AiGenerateHandler),
    );
    registry.register(
        NodeType::AiSummarize,
        Arc::new(AiSummarizeHandler),
    );
    registry.register(
        NodeType::AiClassify,
        Arc::new(AiClassifyHandler),
    );
    registry.register(
        NodeType::AiExtract,
        Arc::new(AiExtractHandler),
    );
    
    // Real Estate Intelligence Nodes
    registry.register(
        NodeType::LogicMatch,
        Arc::new(SmartMatchHandler),
    );
    registry.register(
        NodeType::LogicGeoFence,
        Arc::new(GeoFenceHandler),
    );
    registry.register(
        NodeType::TriggerStateChange,
        Arc::new(StateChangeHandler),
    );
    registry.register(
        NodeType::ActionWhatsapp,
        Arc::new(WhatsappHandler),
    );
    registry.register(
        NodeType::AiContextAware,
        Arc::new(ContextAwareAiHandler),
    );
    
    // Register ScriptNode handler for WASM execution (backend only - requires extism/wasmtime)
    #[cfg(feature = "backend")]
    registry.register(
        NodeType::ScriptNode,
        Arc::new(crate::script_node::ScriptNodeHandler::new()),
    );
    
    registry
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

/// Condition If handler - evaluates conditions on record data
/// Supports: equals, not_equals, greater_than, less_than, >=, <=, contains, is_null, changed_to
pub struct ConditionIfHandler;

#[async_trait]
impl NodeHandler for ConditionIfHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let condition_field = node.config.get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let operator = node.config.get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("equals");
        
        let compare_value = node.config.get("value").cloned().unwrap_or(Value::Null);
        
        // Get current value from input data
        let input_value = inputs.get("data")
            .and_then(|v| v.get(condition_field))
            .cloned()
            .unwrap_or(Value::Null);
        
        // Get old value for change detection (from trigger context)
        let old_value = context.values.get("$trigger")
            .and_then(|t: &serde_json::Value| t.get("old_values"))
            .and_then(|old: &serde_json::Value| old.get(condition_field))
            .cloned();
        
        let result = match operator {
            // Equality operators
            "equals" | "eq" | "==" => input_value == compare_value,
            "not_equals" | "neq" | "!=" => input_value != compare_value,
            
            // Numeric comparison operators
            "greater_than" | "gt" | ">" => {
                compare_numeric(&input_value, &compare_value, |a, b| a > b)
            }
            "less_than" | "lt" | "<" => {
                compare_numeric(&input_value, &compare_value, |a, b| a < b)
            }
            "greater_than_or_equal" | "gte" | ">=" => {
                compare_numeric(&input_value, &compare_value, |a, b| a >= b)
            }
            "less_than_or_equal" | "lte" | "<=" => {
                compare_numeric(&input_value, &compare_value, |a, b| a <= b)
            }
            
            // Changed to - checks if field value changed TO a specific value
            "changed_to" => {
                let changed = if let Some(old) = &old_value {
                    // Value changed FROM something else TO compare_value
                    *old != compare_value && input_value == compare_value
                } else {
                    // No old value means this is a create, check if new value matches
                    input_value == compare_value
                };
                changed
            }
            
            // Changed from - checks if field value changed FROM a specific value
            "changed_from" => {
                if let Some(old) = &old_value {
                    *old == compare_value && input_value != compare_value
                } else {
                    false
                }
            }
            
            // Changed - checks if field value changed at all
            "changed" => {
                if let Some(old) = &old_value {
                    *old != input_value
                } else {
                    true // Created = changed
                }
            }
            
            // Null checks
            "is_null" | "null" => input_value.is_null(),
            "is_not_null" | "not_null" => !input_value.is_null(),
            
            // String operators
            "contains" => {
                input_value.as_str()
                    .map(|s| s.contains(compare_value.as_str().unwrap_or("")))
                    .unwrap_or(false)
            }
            "starts_with" => {
                input_value.as_str()
                    .map(|s| s.starts_with(compare_value.as_str().unwrap_or("")))
                    .unwrap_or(false)
            }
            "ends_with" => {
                input_value.as_str()
                    .map(|s| s.ends_with(compare_value.as_str().unwrap_or("")))
                    .unwrap_or(false)
            }
            
            // In list
            "in" => {
                if let Some(arr) = compare_value.as_array() {
                    arr.contains(&input_value)
                } else {
                    false
                }
            }
            "not_in" => {
                if let Some(arr) = compare_value.as_array() {
                    !arr.contains(&input_value)
                } else {
                    true
                }
            }
            
            _ => false,
        };
        
        tracing::debug!(
            field = condition_field,
            operator = operator,
            result = result,
            "Condition evaluated"
        );
        
        Ok(serde_json::json!({
            "condition": result,
            "field": condition_field,
            "operator": operator,
            "current_value": input_value,
            "compare_value": compare_value,
            "old_value": old_value
        }))
    }
}

/// Helper function for numeric comparisons
fn compare_numeric<F>(a: &Value, b: &Value, cmp: F) -> bool 
where
    F: Fn(f64, f64) -> bool
{
    let a_num = value_to_f64(a);
    let b_num = value_to_f64(b);
    
    match (a_num, b_num) {
        (Some(a), Some(b)) => cmp(a, b),
        _ => false,
    }
}

/// Convert JSON value to f64 for numeric comparison
fn value_to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
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
                    .and_then(|t: &serde_json::Value| t.get("record_id"))
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
                    .and_then(|t: &serde_json::Value| t.get("record_id"))
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

/// AI Generate handler - generates text using LLM
pub struct AiGenerateHandler;

#[async_trait]
impl NodeHandler for AiGenerateHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let prompt_template = node.config.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let system_prompt = node.config.get("system_prompt")
            .and_then(|v| v.as_str());

        // Simple template substitution: {{variable}}
        let mut prompt = prompt_template.to_string();
        for (key, val) in &inputs {
            let placeholder = format!("{{{{{}}}}}", key);
            let val_str = match val {
                Value::String(s) => s.clone(),
                _ => val.to_string(),
            };
            prompt = prompt.replace(&placeholder, &val_str);
        }
        
        // Check for variables in context (like trigger data)
        if let Some(trigger_data) = context.values.get("$trigger") {
             if let Value::Object(map) = trigger_data {
                for (key, val) in map {
                    let placeholder = format!("{{{{trigger.{}}}}}", key);
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        _ => val.to_string(),
                    };
                    prompt = prompt.replace(&placeholder, &val_str);
                }
             }
        }
        
        tracing::info!(prompt = ?prompt, "Executing AI generation");

        if let Some(ai_service) = &context.ai_service {
            match ai_service.generate(&prompt, system_prompt).await {
                Ok(generated_text) => {
                    Ok(serde_json::json!({
                        "text": generated_text,
                        "generated": true
                    }))
                },
                Err(e) => {
                    tracing::error!(error = %e, "AI generation failed");
                    Err(NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: format!("AI service error: {}", e),
                    })
                }
            }
        } else {
            tracing::warn!("No AI service configured in context");
            Ok(serde_json::json!({
                "text": "AI Service not configured",
                "generated": false,
                "mock": true
            }))
        }
    }
}

// ============ External Integration Nodes ============

/// Send SMS handler - integrates with Twilio/WhatsApp
pub struct SendSmsHandler;

#[async_trait]
impl NodeHandler for SendSmsHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get SMS configuration
        let to = node.config.get("to")
            .or_else(|| inputs.get("phone"))
            .cloned()
            .unwrap_or(Value::Null);
        
        let message_template = node.config.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let provider = node.config.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("twilio"); // twilio, whatsapp, vonage
        
        // Template substitution
        let mut message = message_template.to_string();
        for (key, val) in &inputs {
            let placeholder = format!("{{{{{}}}}}", key);
            let val_str = match val {
                Value::String(s) => s.clone(),
                _ => val.to_string(),
            };
            message = message.replace(&placeholder, &val_str);
        }
        
        tracing::info!(
            provider = provider,
            to = ?to,
            message_len = message.len(),
            "Sending SMS/Message"
        );
        
        // TODO: Actual integration with messaging service
        // Would call twilio.rs or whatsapp.rs from jobs-runner
        
        Ok(serde_json::json!({
            "action": "send_sms",
            "provider": provider,
            "to": to,
            "message": message,
            "sent": true,
            "message_id": uuid::Uuid::new_v4().to_string()
        }))
    }
}

/// Webhook handler - sends data to external URLs
pub struct WebhookHandler;

#[async_trait]
impl NodeHandler for WebhookHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let url = node.config.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::NodeExecutionFailed {
                node_id: node.id,
                message: "Webhook URL is required".to_string(),
            })?;
        
        let method = node.config.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST")
            .to_uppercase();
        
        let headers: HashMap<String, String> = node.config.get("headers")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        
        // Build payload from inputs or config
        let payload = node.config.get("payload")
            .cloned()
            .or_else(|| inputs.get("data").cloned())
            .unwrap_or_else(|| serde_json::json!(inputs));
        
        let timeout_seconds = node.config.get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);
        
        tracing::info!(
            url = url,
            method = method.as_str(),
            payload_size = payload.to_string().len(),
            "Executing webhook"
        );
        
        // In production, this would use reqwest or similar HTTP client
        // For now, we log and return mock success
        
        Ok(serde_json::json!({
            "action": "webhook",
            "url": url,
            "method": method,
            "headers": headers,
            "payload": payload,
            "timeout": timeout_seconds,
            "executed": true,
            "response": {
                "status": 200,
                "body": "Mock response"
            }
        }))
    }
}

// ============ AI-Powered Nodes ============

/// AI Summarization handler - summarizes text or entity data
pub struct AiSummarizeHandler;

#[async_trait]
impl NodeHandler for AiSummarizeHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let text = inputs.get("text")
            .or_else(|| inputs.get("data"))
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => serde_json::to_string_pretty(v).unwrap_or_default(),
            })
            .unwrap_or_default();
        
        let max_length = node.config.get("max_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        
        let style = node.config.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("concise"); // concise, detailed, bullet_points
        
        let prompt = format!(
            "Summarize the following text in {} style, max {} words:\n\n{}",
            style, max_length, text
        );
        
        tracing::info!(text_len = text.len(), style = style, "Summarizing text");
        
        if let Some(ai_service) = &context.ai_service {
            match ai_service.generate(&prompt, Some("You are a helpful summarization assistant.")).await {
                Ok(summary) => {
                    Ok(serde_json::json!({
                        "summary": summary,
                        "original_length": text.len(),
                        "summarized": true
                    }))
                },
                Err(e) => {
                    Err(NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: format!("Summarization failed: {}", e),
                    })
                }
            }
        } else {
            Ok(serde_json::json!({
                "summary": format!("[Mock summary of {} chars]", text.len()),
                "summarized": false,
                "mock": true
            }))
        }
    }
}

/// AI Classification handler - classifies text/leads
pub struct AiClassifyHandler;

#[async_trait]
impl NodeHandler for AiClassifyHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let text = inputs.get("text")
            .or_else(|| inputs.get("data"))
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => serde_json::to_string_pretty(v).unwrap_or_default(),
            })
            .unwrap_or_default();
        
        let categories: Vec<String> = node.config.get("categories")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| vec!["hot".to_string(), "warm".to_string(), "cold".to_string()]);
        
        let prompt = format!(
            "Classify the following into one of these categories: {:?}\n\nText: {}\n\nRespond with only the category name.",
            categories, text
        );
        
        tracing::info!(text_len = text.len(), categories = ?categories, "Classifying");
        
        if let Some(ai_service) = &context.ai_service {
            match ai_service.generate(&prompt, Some("You are a classification assistant. Respond with only the category name.")).await {
                Ok(category) => {
                    let category = category.trim().to_string();
                    let confidence = if categories.contains(&category) { 0.9 } else { 0.5 };
                    
                    Ok(serde_json::json!({
                        "category": category,
                        "confidence": confidence,
                        "available_categories": categories,
                        "classified": true
                    }))
                },
                Err(e) => {
                    Err(NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: format!("Classification failed: {}", e),
                    })
                }
            }
        } else {
            Ok(serde_json::json!({
                "category": categories.first().cloned().unwrap_or_default(),
                "confidence": 0.0,
                "classified": false,
                "mock": true
            }))
        }
    }
}

/// AI Data Extraction handler - extracts structured data from text
pub struct AiExtractHandler;

#[async_trait]
impl NodeHandler for AiExtractHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let text = inputs.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let fields: Vec<String> = node.config.get("fields")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| vec!["name".to_string(), "email".to_string(), "phone".to_string()]);
        
        let prompt = format!(
            "Extract the following fields from the text: {:?}\n\nText: {}\n\nRespond in JSON format.",
            fields, text
        );
        
        tracing::info!(text_len = text.len(), fields = ?fields, "Extracting data");
        
        if let Some(ai_service) = &context.ai_service {
            match ai_service.generate(&prompt, Some("You are a data extraction assistant. Always respond in valid JSON.")).await {
                Ok(extracted) => {
                    // Try to parse as JSON
                    let data: Value = serde_json::from_str(&extracted)
                        .unwrap_or_else(|_| serde_json::json!({ "raw": extracted }));
                    
                    Ok(serde_json::json!({
                        "extracted": data,
                        "fields": fields,
                        "success": true
                    }))
                },
                Err(e) => {
                    Err(NodeEngineError::NodeExecutionFailed {
                        node_id: node.id,
                        message: format!("Extraction failed: {}", e),
                    })
                }
            }
        } else {
            Ok(serde_json::json!({
                "extracted": {},
                "fields": fields,
                "success": false,
                "mock": true
            }))
        }
    }
}

/// Delay handler - pauses workflow execution
pub struct DelayHandler;

#[async_trait]
impl NodeHandler for DelayHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let delay_seconds = node.config.get("seconds")
            .or_else(|| inputs.get("seconds"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let delay_minutes = node.config.get("minutes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let total_seconds = delay_seconds + (delay_minutes * 60);
        
        tracing::info!(delay_seconds = total_seconds, "Delaying workflow execution");
        
        // In production, this would schedule a delayed continuation
        // For short delays on backend, use tokio sleep
        #[cfg(feature = "backend")]
        if total_seconds > 0 && total_seconds <= 60 {
            tokio::time::sleep(std::time::Duration::from_secs(total_seconds)).await;
        }
        
        // On WASM/frontend, we just record the delay intent
        #[cfg(not(feature = "backend"))]
        let _ = total_seconds; // Acknowledge the variable for WASM builds
        
        Ok(serde_json::json!({
            "action": "delay",
            "delayed_seconds": total_seconds,
            "completed": true
        }))
    }
}

// ============================================================================
// REAL ESTATE INTELLIGENCE HANDLERS
// ============================================================================

/// Smart Match Handler - The "Tinder for Real Estate" Logic
/// Matches a Source Record (e.g., Tenant Requirements) with Target Records (Properties)
/// based on overlapping fields (Price, Bedrooms, Location).
pub struct SmartMatchHandler;

#[async_trait]
impl NodeHandler for SmartMatchHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let strategy = node.config.get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("strict");
        
        let target_entity = node.config.get("target_entity")
            .and_then(|v| v.as_str())
            .unwrap_or("property");
        
        let threshold = node.config.get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);
        
        // Get matching criteria from inputs or config
        let criteria = inputs.get("criteria")
            .or_else(|| node.config.get("criteria"))
            .cloned()
            .unwrap_or(serde_json::json!({}));
        
        tracing::info!(
            target_entity = target_entity, 
            strategy = strategy,
            threshold = threshold,
            "Executing Smart Match"
        );
        
        // In production: Query pgvector for semantic search or SQL for exact match
        // For now, we simulate the matching logic
        // TODO: Implement actual matching against database
        
        Ok(serde_json::json!({
            "action": "smart_match",
            "target_entity": target_entity,
            "strategy": strategy,
            "threshold": threshold,
            "matches_found": 3, // Mock
            "top_match": {
                "id": uuid::Uuid::new_v4(),
                "score": 0.98,
                "reason": "Perfect match on Location and Budget"
            },
            "all_matches": [],
            "status": "success"
        }))
    }
}

/// GeoFence Handler - Validates if coordinates are within a target zone
pub struct GeoFenceHandler;

#[async_trait]
impl NodeHandler for GeoFenceHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get point to check
        let lat = inputs.get("latitude")
            .or_else(|| node.config.get("latitude"))
            .and_then(|v| v.as_f64());
        
        let lng = inputs.get("longitude")
            .or_else(|| node.config.get("longitude"))
            .and_then(|v| v.as_f64());
        
        // Get target zone center
        let center_lat = node.config.get("center_latitude")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let center_lng = node.config.get("center_longitude")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        // Radius in kilometers
        let radius_km = node.config.get("radius_km")
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);
        
        let (is_within, distance) = match (lat, lng) {
            (Some(lat), Some(lng)) => {
                // Haversine formula for distance calculation
                let distance = haversine_distance(lat, lng, center_lat, center_lng);
                (distance <= radius_km, distance)
            }
            _ => (false, 0.0)
        };
        
        tracing::info!(
            is_within = is_within,
            distance_km = distance,
            radius_km = radius_km,
            "GeoFence check completed"
        );
        
        Ok(serde_json::json!({
            "action": "geofence_check",
            "is_within": is_within,
            "distance_km": distance,
            "radius_km": radius_km,
            "center": {
                "latitude": center_lat,
                "longitude": center_lng
            }
        }))
    }
}

/// Haversine formula to calculate distance between two points on Earth
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    
    let a = (d_lat / 2.0).sin().powi(2) 
        + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    
    EARTH_RADIUS_KM * c
}

/// State Change Handler - Fires when an entity transitions between states
pub struct StateChangeHandler;

#[async_trait]
impl NodeHandler for StateChangeHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get state field to monitor
        let state_field = node.config.get("state_field")
            .and_then(|v| v.as_str())
            .unwrap_or("status");
        
        // Get allowed transitions (optional validation)
        let allowed_from = node.config.get("from")
            .and_then(|v| v.as_str());
        
        let allowed_to = node.config.get("to")
            .and_then(|v| v.as_str());
        
        // Get current and previous values from context
        let trigger_data = context.values.get("$trigger")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        
        let current_value = trigger_data.get(state_field);
        let previous_value = trigger_data.get(&format!("_prev_{}", state_field));
        
        // Check if transition is valid
        let from_valid = allowed_from.map_or(true, |from| {
            previous_value.and_then(|v| v.as_str()) == Some(from)
        });
        
        let to_valid = allowed_to.map_or(true, |to| {
            current_value.and_then(|v| v.as_str()) == Some(to)
        });
        
        let is_valid_transition = from_valid && to_valid;
        
        tracing::info!(
            state_field = state_field,
            from = ?previous_value,
            to = ?current_value,
            is_valid = is_valid_transition,
            "State change detected"
        );
        
        if !is_valid_transition {
            return Err(NodeEngineError::NodeExecutionFailed {
                node_id: node.id,
                message: format!(
                    "Invalid state transition: {:?} -> {:?} (expected {:?} -> {:?})",
                    previous_value, current_value, allowed_from, allowed_to
                ),
            });
        }
        
        Ok(serde_json::json!({
            "action": "state_change",
            "field": state_field,
            "from": previous_value,
            "to": current_value,
            "is_valid": is_valid_transition
        }))
    }
}

/// WhatsApp Handler - Sends template messages via Meta Cloud API
pub struct WhatsappHandler;

#[async_trait]
impl NodeHandler for WhatsappHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let to = node.config.get("to")
            .or_else(|| inputs.get("phone"))
            .or_else(|| inputs.get("to"))
            .cloned()
            .unwrap_or(Value::Null);
        
        let template = node.config.get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("hello_world");
        
        let variables = node.config.get("variables")
            .cloned()
            .unwrap_or(serde_json::json!([]));
        
        tracing::info!(
            to = ?to, 
            template = template, 
            "Sending WhatsApp Business Message"
        );
        
        // In production, this would call the Meta WhatsApp Cloud API
        // For now, we log and return mock response
        // TODO: Integrate with MessagingProvider from core-integrations
        
        Ok(serde_json::json!({
            "action": "send_whatsapp",
            "to": to,
            "template": template,
            "variables": variables,
            "sent": true,
            "provider": "meta_cloud_api",
            "message_id": uuid::Uuid::new_v4().to_string()
        }))
    }
}

/// Context-Aware AI Handler - Uses RAG for system-aware insights
pub struct ContextAwareAiHandler;

#[async_trait]
impl NodeHandler for ContextAwareAiHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let query = inputs.get("query")
            .or_else(|| node.config.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let system_prompt = node.config.get("system_prompt")
            .and_then(|v| v.as_str());
        
        let include_context = node.config.get("include_context")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        tracing::info!(
            query_len = query.len(),
            include_context = include_context,
            "Executing Context-Aware AI"
        );
        
        // In production:
        // 1. Vector search for relevant context (pgvector)
        // 2. Inject context into system prompt
        // 3. Generate with OpenAI/Anthropic
        // TODO: Integrate with EmbeddingService and AiService
        
        Ok(serde_json::json!({
            "action": "ai_context_aware",
            "query": query,
            "response": "This is a mock RAG response. In production, this would include relevant context from your data.",
            "sources": 0,
            "mock": true
        }))
    }
}
