//! Twilio webhook handler for SMS and Voice

use bytes::Bytes;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::{Provider, SystemEvent};
use crate::webhook::{WebhookError, WebhookHandler};

/// Twilio webhook handler
pub struct TwilioHandler {
    /// Base URL of our API for signature validation
    pub base_url: String,
}

impl TwilioHandler {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Parse URL-encoded form data from Twilio
    fn parse_form_data(payload: &[u8]) -> Result<HashMap<String, String>, WebhookError> {
        let body_str = std::str::from_utf8(payload)
            .map_err(|e| WebhookError::ParseError(format!("Invalid UTF-8: {}", e)))?;
        
        let params: HashMap<String, String> = url::form_urlencoded::parse(body_str.as_bytes())
            .into_owned()
            .collect();
        
        Ok(params)
    }

    /// Compute Twilio signature for validation
    /// https://www.twilio.com/docs/usage/security#validating-requests
    fn compute_signature(url: &str, params: &HashMap<String, String>, auth_token: &str) -> String {
        // Sort parameters by key and concatenate
        let mut sorted_keys: Vec<&String> = params.keys().collect();
        sorted_keys.sort();
        
        let mut data = url.to_string();
        for key in sorted_keys {
            data.push_str(key);
            data.push_str(params.get(key).unwrap_or(&String::new()));
        }
        
        // HMAC-SHA1 (Twilio uses SHA1)
        type HmacSha1 = hmac::Hmac<sha1::Sha1>;
        let mut mac = HmacSha1::new_from_slice(auth_token.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        
        base64::encode(mac.finalize().into_bytes())
    }
}

impl WebhookHandler for TwilioHandler {
    fn provider(&self) -> Provider {
        Provider::Twilio
    }

    fn verify_signature(
        &self,
        payload: &[u8],
        headers: &HeaderMap,
        secret: &str,
    ) -> bool {
        // Get Twilio signature from header
        let signature = match headers.get("X-Twilio-Signature") {
            Some(sig) => match sig.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return false,
            },
            None => {
                warn!("Missing X-Twilio-Signature header");
                return false;
            }
        };

        // Parse form data to compute signature
        let params = match Self::parse_form_data(payload) {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Get webhook URL from headers or construct it
        let url = headers
            .get("X-Original-URL")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.base_url.clone());

        // Compute expected signature
        let expected = Self::compute_signature(&url, &params, secret);
        
        // Constant-time comparison
        signature == expected
    }

    fn handle_webhook(
        &self,
        tenant_id: Uuid,
        payload: Bytes,
        headers: &HeaderMap,
    ) -> Result<Vec<SystemEvent>, WebhookError> {
        let params = Self::parse_form_data(&payload)?;
        
        // Determine message type
        let message_type = params.get("MessageSid")
            .map(|_| "sms")
            .or_else(|| params.get("CallSid").map(|_| "voice"))
            .unwrap_or("unknown");

        info!(tenant_id = %tenant_id, message_type = message_type, "Processing Twilio webhook");

        match message_type {
            "sms" => {
                // Incoming SMS
                let from = params.get("From")
                    .ok_or_else(|| WebhookError::MissingField("From".to_string()))?
                    .clone();
                let to = params.get("To")
                    .ok_or_else(|| WebhookError::MissingField("To".to_string()))?
                    .clone();
                let body = params.get("Body")
                    .cloned()
                    .unwrap_or_default();
                let message_sid = params.get("MessageSid")
                    .cloned()
                    .unwrap_or_default();

                Ok(vec![SystemEvent::InteractionCreated {
                    tenant_id,
                    from,
                    to,
                    body,
                    provider: Provider::Twilio,
                    external_id: message_sid,
                }])
            }
            "voice" => {
                // Incoming call - just log for now
                let from = params.get("From")
                    .cloned()
                    .unwrap_or_default();
                let to = params.get("To")
                    .cloned()
                    .unwrap_or_default();
                let call_sid = params.get("CallSid")
                    .cloned()
                    .unwrap_or_default();

                Ok(vec![SystemEvent::InteractionCreated {
                    tenant_id,
                    from,
                    to,
                    body: "[Incoming Call]".to_string(),
                    provider: Provider::Twilio,
                    external_id: call_sid,
                }])
            }
            _ => {
                // Unknown type - return raw webhook
                Ok(vec![SystemEvent::WebhookReceived {
                    tenant_id,
                    provider: Provider::Twilio,
                    payload: serde_json::to_value(&params)
                        .unwrap_or(serde_json::Value::Null),
                }])
            }
        }
    }
}

// Need sha1 for Twilio signature
use sha1::Sha1;
