//! Models for integration configurations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported integration providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Twilio,
    Facebook,
    WhatsApp,
    Email,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Twilio => "twilio",
            Provider::Facebook => "facebook",
            Provider::WhatsApp => "whatsapp",
            Provider::Email => "email",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "twilio" => Some(Provider::Twilio),
            "facebook" => Some(Provider::Facebook),
            "whatsapp" => Some(Provider::WhatsApp),
            "email" => Some(Provider::Email),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Provider::Twilio => "Twilio (SMS/Voice)",
            Provider::Facebook => "Facebook Lead Ads",
            Provider::WhatsApp => "WhatsApp Business",
            Provider::Email => "Email (SMTP)",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Provider::Twilio => "📱",
            Provider::Facebook => "📘",
            Provider::WhatsApp => "💬",
            Provider::Email => "📧",
        }
    }
}

/// Integration configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub provider: Provider,
    pub is_enabled: bool,
    /// Encrypted JSON containing provider-specific credentials
    #[serde(skip_serializing)]
    pub credentials_encrypted: Option<Vec<u8>>,
    /// Webhook secret for signature validation
    #[serde(skip_serializing)]
    pub webhook_secret: String,
    /// Generated webhook URL for this integration
    pub webhook_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status information for an integration (for UI display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub provider: Provider,
    pub display_name: String,
    pub icon: String,
    pub is_configured: bool,
    pub is_enabled: bool,
    pub webhook_url: Option<String>,
    pub last_webhook_at: Option<DateTime<Utc>>,
}

/// Twilio-specific credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioCredentials {
    pub account_sid: String,
    pub auth_token: String,
    pub phone_number: String,
}

/// Facebook-specific credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookCredentials {
    pub app_id: String,
    pub app_secret: String,
    pub page_access_token: String,
    pub page_id: String,
}

/// WhatsApp Business credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppCredentials {
    pub phone_number_id: String,
    pub access_token: String,
    pub verify_token: String,
}

/// Email/SMTP credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCredentials {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

/// Generic credentials wrapper for API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "credentials")]
pub enum ProviderCredentials {
    #[serde(rename = "twilio")]
    Twilio(TwilioCredentials),
    #[serde(rename = "facebook")]
    Facebook(FacebookCredentials),
    #[serde(rename = "whatsapp")]
    WhatsApp(WhatsAppCredentials),
    #[serde(rename = "email")]
    Email(EmailCredentials),
}

impl ProviderCredentials {
    pub fn provider(&self) -> Provider {
        match self {
            ProviderCredentials::Twilio(_) => Provider::Twilio,
            ProviderCredentials::Facebook(_) => Provider::Facebook,
            ProviderCredentials::WhatsApp(_) => Provider::WhatsApp,
            ProviderCredentials::Email(_) => Provider::Email,
        }
    }
}

/// System events emitted from webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// New SMS/Voice interaction received
    InteractionCreated {
        tenant_id: Uuid,
        from: String,
        to: String,
        body: String,
        provider: Provider,
        external_id: String,
    },
    /// New contact created from lead form
    ContactCreated {
        tenant_id: Uuid,
        email: Option<String>,
        phone: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        source: String,
        external_id: String,
        raw_data: serde_json::Value,
    },
    /// Webhook received but not processed
    WebhookReceived {
        tenant_id: Uuid,
        provider: Provider,
        payload: serde_json::Value,
    },
}
