//! Notification Handlers - Production SMS and Email
//!
//! Provides:
//! - SendSmsHandler with Twilio
//! - SendEmailHandler with SendGrid
//! - Notification logging and tracking

use async_trait::async_trait;
use core_models::NodeDef;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::context::ExecutionContext;
use crate::NodeEngineError;
use crate::nodes::NodeHandler;

// ============================================================================
// SMS - Twilio Integration
// ============================================================================

/// Twilio SMS configuration
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
}

impl TwilioConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            account_sid: std::env::var("TWILIO_ACCOUNT_SID").ok()?,
            auth_token: std::env::var("TWILIO_AUTH_TOKEN").ok()?,
            from_number: std::env::var("TWILIO_FROM_NUMBER").ok()?,
        })
    }
}

/// SMS send result
#[derive(Debug, Serialize, Deserialize)]
pub struct SmsSendResult {
    pub message_sid: String,
    pub status: String,
    pub to: String,
}

/// Twilio SMS Handler for workflow engine
pub struct SendSmsHandler {
    config: TwilioConfig,
    client: reqwest::Client,
}

impl SendSmsHandler {
    pub fn new(config: TwilioConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
    
    pub fn from_env() -> Option<Self> {
        TwilioConfig::from_env().map(Self::new)
    }
    
    async fn send_sms(&self, to: &str, body: &str) -> Result<SmsSendResult, String> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.config.account_sid
        );
        
        let form = [
            ("To", to),
            ("From", &self.config.from_number),
            ("Body", body),
        ];
        
        let response = self.client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&form)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await
                .unwrap_or(json!({"message": "Unknown error"}));
            return Err(format!("Twilio error: {:?}", error["message"]));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        Ok(SmsSendResult {
            message_sid: json["sid"].as_str().unwrap_or("").to_string(),
            status: json["status"].as_str().unwrap_or("queued").to_string(),
            to: to.to_string(),
        })
    }
}

#[async_trait]
impl NodeHandler for SendSmsHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get phone number
        let phone_field = node.config.get("phone_field")
            .and_then(|v| v.as_str())
            .unwrap_or("phone");
        
        let phone = node.config.get("phone")
            .or(inputs.get("phone"))
            .or(inputs.get("to"))
            .or_else(|| context.trigger_data.get(phone_field))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing phone number".into()))?;
        
        // Get message body
        let body = node.config.get("body")
            .or(inputs.get("message"))
            .or(inputs.get("body"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing message body".into()))?;
        
        // Template variable substitution
        let body = substitute_variables(body, &inputs, &context.trigger_data);
        
        // Send SMS
        let result = self.send_sms(phone, &body).await
            .map_err(|e| NodeEngineError::ExecutionFailed(e))?;
        
        Ok(json!({
            "success": true,
            "message_sid": result.message_sid,
            "status": result.status,
            "to": result.to,
            "channel": "sms",
            "provider": "twilio",
        }))
    }
}

// ============================================================================
// Email - SendGrid Integration
// ============================================================================

/// SendGrid Email configuration
pub struct SendGridConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: Option<String>,
}

impl SendGridConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            api_key: std::env::var("SENDGRID_API_KEY").ok()?,
            from_email: std::env::var("SENDGRID_FROM_EMAIL").ok()?,
            from_name: std::env::var("SENDGRID_FROM_NAME").ok(),
        })
    }
}

/// Email send result
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSendResult {
    pub message_id: Option<String>,
    pub status: String,
    pub to: String,
}

/// SendGrid Email Handler for workflow engine
pub struct SendEmailHandler {
    config: SendGridConfig,
    client: reqwest::Client,
}

impl SendEmailHandler {
    pub fn new(config: SendGridConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
    
    pub fn from_env() -> Option<Self> {
        SendGridConfig::from_env().map(Self::new)
    }
    
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body_html: &str,
        body_text: Option<&str>,
    ) -> Result<EmailSendResult, String> {
        let payload = json!({
            "personalizations": [{
                "to": [{ "email": to }]
            }],
            "from": {
                "email": self.config.from_email,
                "name": self.config.from_name.as_deref().unwrap_or("Jirsi Platform")
            },
            "subject": subject,
            "content": [
                {
                    "type": "text/plain",
                    "value": body_text.unwrap_or(body_html)
                },
                {
                    "type": "text/html",
                    "value": body_html
                }
            ]
        });
        
        let response = self.client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        let status_code = response.status().as_u16();
        
        if status_code != 202 {
            let error: serde_json::Value = response.json().await
                .unwrap_or(json!({"errors": [{"message": "Unknown error"}]}));
            return Err(format!("SendGrid error: {:?}", error));
        }
        
        // SendGrid returns message ID in header
        let message_id = response.headers()
            .get("x-message-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Ok(EmailSendResult {
            message_id,
            status: "accepted".to_string(),
            to: to.to_string(),
        })
    }
}

#[async_trait]
impl NodeHandler for SendEmailHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get email address
        let email_field = node.config.get("email_field")
            .and_then(|v| v.as_str())
            .unwrap_or("email");
        
        let to = node.config.get("to")
            .or(inputs.get("to"))
            .or(inputs.get("email"))
            .or_else(|| context.trigger_data.get(email_field))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing email address".into()))?;
        
        // Get subject
        let subject = node.config.get("subject")
            .or(inputs.get("subject"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing email subject".into()))?;
        
        // Get body
        let body_html = node.config.get("body")
            .or(node.config.get("body_html"))
            .or(inputs.get("body"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing email body".into()))?;
        
        let body_text = node.config.get("body_text")
            .or(inputs.get("body_text"))
            .and_then(|v| v.as_str());
        
        // Template variable substitution
        let subject = substitute_variables(subject, &inputs, &context.trigger_data);
        let body_html = substitute_variables(body_html, &inputs, &context.trigger_data);
        
        // Send email
        let result = self.send_email(to, &subject, &body_html, body_text.as_deref()).await
            .map_err(|e| NodeEngineError::ExecutionFailed(e))?;
        
        Ok(json!({
            "success": true,
            "message_id": result.message_id,
            "status": result.status,
            "to": result.to,
            "channel": "email",
            "provider": "sendgrid",
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Substitute {{variable}} placeholders in text
fn substitute_variables(template: &str, inputs: &HashMap<String, Value>, trigger_data: &Value) -> String {
    let mut result = template.to_string();
    
    // Find all {{variable}} patterns
    let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
    
    for cap in re.captures_iter(template) {
        let full_match = &cap[0];
        let var_name = &cap[1];
        
        // Try inputs first, then trigger_data
        let value = inputs.get(var_name)
            .or_else(|| trigger_data.get(var_name))
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => v.to_string(),
            })
            .unwrap_or_else(|| full_match.to_string());
        
        result = result.replace(full_match, &value);
    }
    
    result
}

/// Mock SMS handler for testing (doesn't send real SMS)
pub struct MockSmsHandler;

#[async_trait]
impl NodeHandler for MockSmsHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let to = inputs.get("to")
            .or(node.config.get("phone"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        tracing::info!(to = %to, "Mock SMS sent");
        
        Ok(json!({
            "success": true,
            "message_sid": format!("SM_mock_{}", uuid::Uuid::new_v4()),
            "status": "mock_sent",
            "to": to,
            "provider": "mock",
        }))
    }
}

/// Mock Email handler for testing
pub struct MockEmailHandler;

#[async_trait]
impl NodeHandler for MockEmailHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let to = inputs.get("to")
            .or(node.config.get("to"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        tracing::info!(to = %to, "Mock email sent");
        
        Ok(json!({
            "success": true,
            "message_id": format!("mock_{}", uuid::Uuid::new_v4()),
            "status": "mock_sent",
            "to": to,
            "provider": "mock",
        }))
    }
}
