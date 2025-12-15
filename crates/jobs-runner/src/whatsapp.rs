//! WhatsApp Worker - Send WhatsApp messages via Twilio API
//!
//! Uses Twilio's WhatsApp Business API to send messages.

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

/// WhatsApp job payload
#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppJobPayload {
    pub recipient: String,        // Phone number with country code (+1234567890)
    pub message: String,          // Text message content
    pub template_id: Option<String>, // Optional template SID
    pub template_params: Option<Vec<String>>, // Template variables
    pub tenant_id: Uuid,
    pub interaction_id: Option<Uuid>,
}

/// Twilio credentials from integrations table
#[derive(Debug)]
struct TwilioCredentials {
    account_sid: String,
    auth_token: String,
    from_number: String, // WhatsApp-enabled Twilio number
}

/// WhatsApp service for sending messages via Twilio
pub struct WhatsAppService {
    pool: PgPool,
    client: Client,
}

impl WhatsAppService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            client: Client::new(),
        }
    }

    /// Fetch Twilio credentials for a tenant
    async fn get_twilio_credentials(&self, tenant_id: Uuid) -> Result<TwilioCredentials> {
        // First try WhatsApp-specific integration
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"
            SELECT credentials_encrypted
            FROM integrations
            WHERE tenant_id = $1 AND provider IN ('whatsapp', 'twilio') AND is_enabled = true
            ORDER BY CASE WHEN provider = 'whatsapp' THEN 0 ELSE 1 END
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((credentials_bytes,)) => {
                // TODO: Decrypt credentials using AES-256-GCM
                let creds: serde_json::Value = serde_json::from_slice(&credentials_bytes)
                    .map_err(|e| anyhow!("Failed to parse Twilio credentials: {}", e))?;

                Ok(TwilioCredentials {
                    account_sid: creds["account_sid"].as_str().unwrap_or("").to_string(),
                    auth_token: creds["auth_token"].as_str().unwrap_or("").to_string(),
                    from_number: creds["phone_number"].as_str().unwrap_or("").to_string(),
                })
            }
            None => Err(anyhow!("No Twilio/WhatsApp integration configured for tenant {}", tenant_id)),
        }
    }

    /// Send a WhatsApp message via Twilio
    pub async fn send_message(&self, payload: &WhatsAppJobPayload) -> Result<()> {
        info!(
            recipient = %payload.recipient,
            tenant_id = %payload.tenant_id,
            "Sending WhatsApp message"
        );

        let creds = self.get_twilio_credentials(payload.tenant_id).await?;

        // Format numbers for WhatsApp (Twilio requires "whatsapp:" prefix)
        let from = format!("whatsapp:{}", creds.from_number);
        let to = format!("whatsapp:{}", payload.recipient);

        // Build request body
        let body = if let Some(template_id) = &payload.template_id {
            // Template message
            vec![
                ("From", from.as_str()),
                ("To", to.as_str()),
                ("ContentSid", template_id.as_str()),
            ]
        } else {
            // Regular text message
            vec![
                ("From", from.as_str()),
                ("To", to.as_str()),
                ("Body", payload.message.as_str()),
            ]
        };

        // Twilio Messages API URL
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            creds.account_sid
        );

        // Send request
        let response = self
            .client
            .post(&url)
            .basic_auth(&creds.account_sid, Some(&creds.auth_token))
            .form(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let message_sid = result["sid"].as_str().unwrap_or("unknown");

            info!(
                recipient = %payload.recipient,
                message_sid = %message_sid,
                "WhatsApp message sent successfully"
            );

            // Update interaction status if provided
            if let Some(interaction_id) = payload.interaction_id {
                self.update_interaction_status(interaction_id, "sent", Some(message_sid)).await?;
            }

            Ok(())
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();

            error!(
                recipient = %payload.recipient,
                status = %status,
                error = %error_body,
                "Failed to send WhatsApp message"
            );

            // Update interaction status to failed
            if let Some(interaction_id) = payload.interaction_id {
                self.update_interaction_status(interaction_id, "failed", None).await?;
            }

            Err(anyhow!("Twilio API error ({}): {}", status, error_body))
        }
    }

    /// Update interaction status after sending
    async fn update_interaction_status(
        &self,
        interaction_id: Uuid,
        status: &str,
        external_id: Option<&str>,
    ) -> Result<()> {
        let metadata = if let Some(sid) = external_id {
            serde_json::json!({
                "delivery_status": status,
                "twilio_sid": sid
            })
        } else {
            serde_json::json!({
                "delivery_status": status
            })
        };

        sqlx::query(
            r#"
            UPDATE interactions
            SET metadata = COALESCE(metadata, '{}') || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(metadata)
        .bind(interaction_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Process a WhatsApp job payload
pub async fn process_whatsapp_job(pool: &PgPool, payload: &serde_json::Value) -> Result<()> {
    let wa_payload: WhatsAppJobPayload = serde_json::from_value(payload.clone())
        .map_err(|e| anyhow!("Invalid WhatsApp job payload: {}", e))?;

    let service = WhatsAppService::new(pool.clone());
    service.send_message(&wa_payload).await
}
