//! Email Worker - SMTP email sending using lettre
//!
//! Sends emails using SMTP credentials stored in the integrations table.

use anyhow::{anyhow, Result};
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

/// Email job payload
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailJobPayload {
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub html_body: Option<String>,
    pub from_name: Option<String>,
    pub tenant_id: Uuid,
    pub interaction_id: Option<Uuid>,
}

/// SMTP credentials from integrations table
#[derive(Debug)]
struct SmtpCredentials {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_email: String,
    from_name: String,
}

/// Email service for sending emails via SMTP
pub struct EmailService {
    pool: PgPool,
}

impl EmailService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Fetch SMTP credentials for a tenant
    async fn get_smtp_credentials(&self, tenant_id: Uuid) -> Result<SmtpCredentials> {
        // Fetch from integrations table
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"
            SELECT credentials_encrypted
            FROM integrations
            WHERE tenant_id = $1 AND provider = 'email' AND is_enabled = true
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((credentials_bytes,)) => {
                // TODO: Decrypt credentials using AES-256-GCM
                // For now, parse as JSON directly (not encrypted in dev)
                let creds: serde_json::Value = serde_json::from_slice(&credentials_bytes)
                    .map_err(|e| anyhow!("Failed to parse SMTP credentials: {}", e))?;

                Ok(SmtpCredentials {
                    host: creds["host"].as_str().unwrap_or("smtp.gmail.com").to_string(),
                    port: creds["port"].as_u64().unwrap_or(587) as u16,
                    username: creds["username"].as_str().unwrap_or("").to_string(),
                    password: creds["password"].as_str().unwrap_or("").to_string(),
                    from_email: creds["from_email"].as_str().unwrap_or("").to_string(),
                    from_name: creds["from_name"].as_str().unwrap_or("Jirsi CRM").to_string(),
                })
            }
            None => Err(anyhow!("No SMTP integration configured for tenant {}", tenant_id)),
        }
    }

    /// Send an email
    pub async fn send_email(&self, payload: &EmailJobPayload) -> Result<()> {
        info!(
            recipient = %payload.recipient,
            subject = %payload.subject,
            tenant_id = %payload.tenant_id,
            "Sending email"
        );

        // Get SMTP credentials
        let creds = self.get_smtp_credentials(payload.tenant_id).await?;

        // Build the email message
        let from_mailbox: Mailbox = format!("{} <{}>", creds.from_name, creds.from_email)
            .parse()
            .map_err(|e| anyhow!("Invalid from address: {}", e))?;

        let to_mailbox: Mailbox = payload
            .recipient
            .parse()
            .map_err(|e| anyhow!("Invalid recipient address: {}", e))?;

        let mut email_builder = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(&payload.subject);

        // Add HTML or plain text body
        let email = if let Some(html) = &payload.html_body {
            email_builder
                .header(ContentType::TEXT_HTML)
                .body(html.clone())?
        } else {
            email_builder
                .header(ContentType::TEXT_PLAIN)
                .body(payload.body.clone())?
        };

        // Create SMTP transport
        let smtp_creds = Credentials::new(creds.username.clone(), creds.password.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&creds.host)?
                .port(creds.port)
                .credentials(smtp_creds)
                .build();

        // Send the email
        match mailer.send(email).await {
            Ok(response) => {
                info!(
                    recipient = %payload.recipient,
                    "Email sent successfully: {:?}",
                    response
                );

                // Update interaction status if provided
                if let Some(interaction_id) = payload.interaction_id {
                    self.update_interaction_status(interaction_id, "sent").await?;
                }

                Ok(())
            }
            Err(e) => {
                error!(
                    recipient = %payload.recipient,
                    error = %e,
                    "Failed to send email"
                );

                // Update interaction status to failed
                if let Some(interaction_id) = payload.interaction_id {
                    self.update_interaction_status(interaction_id, "failed").await?;
                }

                Err(anyhow!("Failed to send email: {}", e))
            }
        }
    }

    /// Update interaction status after sending
    async fn update_interaction_status(&self, interaction_id: Uuid, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE interactions
            SET metadata = jsonb_set(COALESCE(metadata, '{}'), '{delivery_status}', $1::jsonb),
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(serde_json::json!(status))
        .bind(interaction_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Process an email job payload
pub async fn process_email_job(pool: &PgPool, payload: &serde_json::Value) -> Result<()> {
    let email_payload: EmailJobPayload = serde_json::from_value(payload.clone())
        .map_err(|e| anyhow!("Invalid email job payload: {}", e))?;

    let service = EmailService::new(pool.clone());
    service.send_email(&email_payload).await
}
