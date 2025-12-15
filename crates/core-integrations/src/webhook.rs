//! Webhook handler trait and dispatcher

use bytes::Bytes;
use http::HeaderMap;
use uuid::Uuid;

use crate::models::{Provider, SystemEvent};

/// Trait for handling webhooks from external providers
pub trait WebhookHandler: Send + Sync {
    /// Get the provider type
    fn provider(&self) -> Provider;

    /// Verify the webhook signature
    fn verify_signature(
        &self,
        payload: &[u8],
        headers: &HeaderMap,
        secret: &str,
    ) -> bool;

    /// Parse the webhook payload and convert to system event
    fn handle_webhook(
        &self,
        tenant_id: Uuid,
        payload: Bytes,
        headers: &HeaderMap,
    ) -> Result<Vec<SystemEvent>, WebhookError>;
}

/// Errors that can occur during webhook processing
#[derive(Debug)]
pub enum WebhookError {
    /// Invalid signature
    InvalidSignature,
    /// Failed to parse payload
    ParseError(String),
    /// Provider-specific error
    ProviderError(String),
    /// Missing required field
    MissingField(String),
}

impl std::fmt::Display for WebhookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookError::InvalidSignature => write!(f, "Invalid webhook signature"),
            WebhookError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            WebhookError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            WebhookError::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl std::error::Error for WebhookError {}

/// Webhook dispatcher that routes to appropriate handler
pub struct WebhookDispatcher {
    handlers: Vec<Box<dyn WebhookHandler>>,
}

impl WebhookDispatcher {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn register(mut self, handler: Box<dyn WebhookHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    pub fn get_handler(&self, provider: Provider) -> Option<&dyn WebhookHandler> {
        self.handlers
            .iter()
            .find(|h| h.provider() == provider)
            .map(|h| h.as_ref())
    }
}

impl Default for WebhookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
