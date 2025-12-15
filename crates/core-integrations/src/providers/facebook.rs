//! Facebook Lead Ads webhook handler

use bytes::Bytes;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::{Provider, SystemEvent};
use crate::webhook::{WebhookError, WebhookHandler};

/// Facebook webhook handler
pub struct FacebookHandler {
    /// Facebook App Secret for signature verification
    pub verify_token: String,
}

impl FacebookHandler {
    pub fn new(verify_token: String) -> Self {
        Self { verify_token }
    }
}

/// Facebook webhook payload structure
#[derive(Debug, Deserialize, Serialize)]
struct FacebookWebhook {
    object: String,
    entry: Vec<FacebookEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FacebookEntry {
    id: String,
    time: i64,
    changes: Option<Vec<FacebookChange>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FacebookChange {
    field: String,
    value: FacebookLeadgenValue,
}

#[derive(Debug, Deserialize, Serialize)]
struct FacebookLeadgenValue {
    form_id: Option<String>,
    leadgen_id: Option<String>,
    page_id: Option<String>,
    // Lead data (if fetched)
    #[serde(default)]
    field_data: Vec<FacebookFieldData>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FacebookFieldData {
    name: String,
    values: Vec<String>,
}

impl WebhookHandler for FacebookHandler {
    fn provider(&self) -> Provider {
        Provider::Facebook
    }

    fn verify_signature(
        &self,
        payload: &[u8],
        headers: &HeaderMap,
        secret: &str,
    ) -> bool {
        // Get signature from header (X-Hub-Signature-256)
        let signature = match headers.get("X-Hub-Signature-256") {
            Some(sig) => match sig.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return false,
            },
            None => {
                // Try legacy header
                match headers.get("X-Hub-Signature") {
                    Some(sig) => match sig.to_str() {
                        Ok(s) => s.to_string(),
                        Err(_) => return false,
                    },
                    None => {
                        warn!("Missing X-Hub-Signature header");
                        return false;
                    }
                }
            }
        };

        // Signature format: sha256=<hex>
        let expected_prefix = "sha256=";
        if !signature.starts_with(expected_prefix) {
            warn!("Invalid signature format");
            return false;
        }
        let signature_hex = &signature[expected_prefix.len()..];

        // Compute HMAC-SHA256
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(payload);
        let expected = hex::encode(mac.finalize().into_bytes());

        // Constant-time comparison
        signature_hex == expected
    }

    fn handle_webhook(
        &self,
        tenant_id: Uuid,
        payload: Bytes,
        _headers: &HeaderMap,
    ) -> Result<Vec<SystemEvent>, WebhookError> {
        // Parse webhook payload
        let webhook: FacebookWebhook = serde_json::from_slice(&payload)
            .map_err(|e| WebhookError::ParseError(format!("Invalid JSON: {}", e)))?;

        if webhook.object != "page" {
            return Ok(vec![SystemEvent::WebhookReceived {
                tenant_id,
                provider: Provider::Facebook,
                payload: serde_json::from_slice(&payload).unwrap_or(serde_json::Value::Null),
            }]);
        }

        let mut events = Vec::new();

        for entry in webhook.entry {
            if let Some(changes) = entry.changes {
                for change in changes {
                    if change.field == "leadgen" {
                        info!(
                            tenant_id = %tenant_id,
                            leadgen_id = ?change.value.leadgen_id,
                            "Processing Facebook lead"
                        );

                        // Extract field data if available
                        let mut email = None;
                        let mut phone = None;
                        let mut first_name = None;
                        let mut last_name = None;

                        for field in &change.value.field_data {
                            let value = field.values.first().cloned();
                            match field.name.to_lowercase().as_str() {
                                "email" => email = value,
                                "phone_number" | "phone" => phone = value,
                                "first_name" => first_name = value,
                                "last_name" => last_name = value,
                                "full_name" => {
                                    if let Some(name) = value {
                                        let parts: Vec<&str> = name.splitn(2, ' ').collect();
                                        first_name = Some(parts[0].to_string());
                                        last_name = parts.get(1).map(|s| s.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }

                        events.push(SystemEvent::ContactCreated {
                            tenant_id,
                            email,
                            phone,
                            first_name,
                            last_name,
                            source: "facebook_lead_ad".to_string(),
                            external_id: change.value.leadgen_id.clone().unwrap_or_default(),
                            raw_data: serde_json::to_value(&change.value)
                                .unwrap_or(serde_json::Value::Null),
                        });
                    }
                }
            }
        }

        if events.is_empty() {
            // Return raw webhook if no leads
            Ok(vec![SystemEvent::WebhookReceived {
                tenant_id,
                provider: Provider::Facebook,
                payload: serde_json::from_slice(&payload).unwrap_or(serde_json::Value::Null),
            }])
        } else {
            Ok(events)
        }
    }
}

/// Facebook webhook verification (GET request for subscription)
#[derive(Debug, Deserialize)]
pub struct FacebookVerifyRequest {
    #[serde(rename = "hub.mode")]
    pub mode: String,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: String,
    #[serde(rename = "hub.challenge")]
    pub challenge: String,
}

impl FacebookHandler {
    /// Verify Facebook webhook subscription
    pub fn verify_subscription(&self, request: &FacebookVerifyRequest) -> Option<String> {
        if request.mode == "subscribe" && request.verify_token == self.verify_token {
            Some(request.challenge.clone())
        } else {
            None
        }
    }
}
