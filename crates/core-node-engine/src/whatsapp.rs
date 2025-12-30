//! WhatsApp Handler - Meta WhatsApp Cloud API Integration
//!
//! Production WhatsApp messaging using Meta's Cloud API.
//! Supports:
//! - Template messages (pre-approved)
//! - Interactive messages
//! - Media messages
//! - Message status webhooks

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WhatsApp message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WhatsAppMessage {
    /// Template message (pre-approved)
    Template {
        name: String,
        language: String,
        components: Vec<TemplateComponent>,
    },
    /// Plain text message
    Text {
        body: String,
        preview_url: bool,
    },
    /// Interactive message with buttons
    Interactive {
        body: String,
        buttons: Vec<InteractiveButton>,
    },
    /// Media message
    Media {
        media_type: MediaType,
        url: String,
        caption: Option<String>,
    },
}

/// Template component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateComponent {
    #[serde(rename = "type")]
    pub component_type: String, // "header", "body", "button"
    pub parameters: Vec<TemplateParameter>,
}

/// Template parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    #[serde(rename = "type")]
    pub param_type: String, // "text", "currency", "date_time", "image", "document"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<CurrencyValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<MediaValue>,
}

/// Currency value for template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyValue {
    pub fallback_value: String,
    pub code: String,
    pub amount_1000: i64,
}

/// Media value for template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaValue {
    pub link: String,
}

/// Interactive button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveButton {
    #[serde(rename = "type")]
    pub button_type: String, // "reply"
    pub reply: ButtonReply,
}

/// Button reply structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonReply {
    pub id: String,
    pub title: String,
}

/// Media types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
    Document,
    Audio,
}

/// Message send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppSendResult {
    pub message_id: String,
    pub status: String,
}

/// WhatsApp API errors
#[derive(Debug, thiserror::Error)]
pub enum WhatsAppError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid phone number: {0}")]
    InvalidPhone(String),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// WhatsApp provider trait
#[async_trait]
pub trait WhatsAppProvider: Send + Sync {
    /// Send a WhatsApp message
    async fn send_message(&self, to: &str, message: WhatsAppMessage) -> Result<WhatsAppSendResult, WhatsAppError>;
    
    /// Get message status
    async fn get_message_status(&self, message_id: &str) -> Result<String, WhatsAppError>;
    
    /// Mark message as read
    async fn mark_as_read(&self, message_id: &str) -> Result<(), WhatsAppError>;
}

/// Meta WhatsApp Cloud API Provider
pub struct MetaWhatsAppProvider {
    access_token: String,
    phone_number_id: String,
    client: reqwest::Client,
}

impl MetaWhatsAppProvider {
    pub fn new(access_token: String, phone_number_id: String) -> Self {
        Self {
            access_token,
            phone_number_id,
            client: reqwest::Client::new(),
        }
    }
    
    fn api_base(&self) -> String {
        format!("https://graph.facebook.com/v18.0/{}", self.phone_number_id)
    }
    
    fn normalize_phone(&self, phone: &str) -> Result<String, WhatsAppError> {
        // Remove all non-digit characters except +
        let cleaned: String = phone.chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect();
        
        // Ensure it starts with country code
        if cleaned.len() < 10 {
            return Err(WhatsAppError::InvalidPhone(phone.to_string()));
        }
        
        // Remove leading + if present
        let normalized = cleaned.trim_start_matches('+').to_string();
        
        Ok(normalized)
    }
}

#[async_trait]
impl WhatsAppProvider for MetaWhatsAppProvider {
    async fn send_message(&self, to: &str, message: WhatsAppMessage) -> Result<WhatsAppSendResult, WhatsAppError> {
        let phone = self.normalize_phone(to)?;
        
        let body = match &message {
            WhatsAppMessage::Template { name, language, components } => {
                serde_json::json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": phone,
                    "type": "template",
                    "template": {
                        "name": name,
                        "language": {
                            "code": language
                        },
                        "components": components
                    }
                })
            }
            WhatsAppMessage::Text { body, preview_url } => {
                serde_json::json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": phone,
                    "type": "text",
                    "text": {
                        "body": body,
                        "preview_url": preview_url
                    }
                })
            }
            WhatsAppMessage::Interactive { body, buttons } => {
                serde_json::json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": phone,
                    "type": "interactive",
                    "interactive": {
                        "type": "button",
                        "body": { "text": body },
                        "action": {
                            "buttons": buttons
                        }
                    }
                })
            }
            WhatsAppMessage::Media { media_type, url, caption } => {
                let media_type_str = match media_type {
                    MediaType::Image => "image",
                    MediaType::Video => "video",
                    MediaType::Document => "document",
                    MediaType::Audio => "audio",
                };
                
                serde_json::json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": phone,
                    "type": media_type_str,
                    media_type_str: {
                        "link": url,
                        "caption": caption
                    }
                })
            }
        };
        
        let response = self.client
            .post(format!("{}/messages", self.api_base()))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| WhatsAppError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 429 {
            return Err(WhatsAppError::RateLimited);
        }
        
        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await
                .unwrap_or(serde_json::json!({"error": {"message": "Unknown error"}}));
            return Err(WhatsAppError::ApiError(
                error["error"]["message"].as_str().unwrap_or("Unknown error").to_string()
            ));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| WhatsAppError::NetworkError(e.to_string()))?;
        
        let message_id = json["messages"][0]["id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        Ok(WhatsAppSendResult {
            message_id,
            status: "sent".to_string(),
        })
    }
    
    async fn get_message_status(&self, message_id: &str) -> Result<String, WhatsAppError> {
        // Note: Meta doesn't have a direct status API; status comes via webhooks
        // This is a placeholder that would check a local cache/database
        Ok("delivered".to_string())
    }
    
    async fn mark_as_read(&self, message_id: &str) -> Result<(), WhatsAppError> {
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });
        
        let response = self.client
            .post(format!("{}/messages", self.api_base()))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| WhatsAppError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(WhatsAppError::ApiError("Failed to mark as read".to_string()));
        }
        
        Ok(())
    }
}

// ============================================================================
// WhatsApp Workflow Node Handler
// ============================================================================

use core_models::NodeDef;
use serde_json::{json, Value};
use crate::context::ExecutionContext;
use crate::NodeEngineError;
use crate::nodes::NodeHandler;

/// WhatsApp Action Handler for workflow engine
pub struct ActionWhatsAppHandler {
    provider: std::sync::Arc<dyn WhatsAppProvider>,
}

impl ActionWhatsAppHandler {
    pub fn new(provider: std::sync::Arc<dyn WhatsAppProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl NodeHandler for ActionWhatsAppHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get recipient phone number
        let phone = node.config.get("phone")
            .or(inputs.get("phone"))
            .or(inputs.get("to"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing phone number".into()))?;
        
        // Get message type
        let message_type = node.config.get("message_type")
            .and_then(|v| v.as_str())
            .unwrap_or("template");
        
        let message = match message_type {
            "template" => {
                let template_name = node.config.get("template_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| NodeEngineError::InvalidConfig("Missing template_name".into()))?;
                
                let language = node.config.get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("en");
                
                // Build components from config/inputs
                let mut components = Vec::new();
                
                // Body parameters
                if let Some(body_params) = node.config.get("body_parameters")
                    .or(inputs.get("body_parameters"))
                    .and_then(|v| v.as_array()) 
                {
                    let parameters: Vec<TemplateParameter> = body_params.iter()
                        .filter_map(|p| p.as_str().map(|s| TemplateParameter {
                            param_type: "text".to_string(),
                            text: Some(s.to_string()),
                            currency: None,
                            image: None,
                        }))
                        .collect();
                    
                    if !parameters.is_empty() {
                        components.push(TemplateComponent {
                            component_type: "body".to_string(),
                            parameters,
                        });
                    }
                }
                
                WhatsAppMessage::Template {
                    name: template_name.to_string(),
                    language: language.to_string(),
                    components,
                }
            }
            "text" => {
                let body = node.config.get("body")
                    .or(inputs.get("message"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| NodeEngineError::InvalidConfig("Missing message body".into()))?;
                
                WhatsAppMessage::Text {
                    body: body.to_string(),
                    preview_url: false,
                }
            }
            _ => {
                return Err(NodeEngineError::InvalidConfig(
                    format!("Unknown message type: {}", message_type)
                ));
            }
        };
        
        // Send message
        let result = self.provider.send_message(phone, message).await
            .map_err(|e| NodeEngineError::ExecutionFailed(e.to_string()))?;
        
        Ok(json!({
            "success": true,
            "message_id": result.message_id,
            "status": result.status,
            "phone": phone,
        }))
    }
}

/// Pre-approved WhatsApp templates for common use cases
pub mod templates {
    /// Template for viewing confirmation
    pub const VIEWING_CONFIRMATION: &str = "viewing_confirmation";
    
    /// Template for payment reminder
    pub const PAYMENT_REMINDER: &str = "payment_reminder";
    
    /// Template for contract signed notification
    pub const CONTRACT_SIGNED: &str = "contract_signed";
    
    /// Template for new property match alert
    pub const PROPERTY_MATCH_ALERT: &str = "property_match_alert";
    
    /// Template for lead follow-up
    pub const LEAD_FOLLOWUP: &str = "lead_followup";
}
