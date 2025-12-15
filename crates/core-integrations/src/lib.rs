//! Core Integrations - Third-party API integrations and Webhook Gateway
//!
//! Provides secure credential storage and webhook handling for external services
//! like Twilio, Facebook, WhatsApp, and Email.

pub mod models;
pub mod service;
pub mod webhook;
pub mod providers;
pub mod encryption;

pub use models::{IntegrationConfig, Provider, ProviderStatus};
pub use service::IntegrationService;
pub use webhook::WebhookHandler;
