//! Payment Provider - Stripe Connect Integration
//!
//! Provides:
//! - PaymentProvider trait for payment processing
//! - StripeConnectProvider implementation
//! - ActionCollectPayment workflow node

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment amount with currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAmount {
    /// Amount in smallest currency unit (cents for USD)
    pub amount: i64,
    /// ISO 4217 currency code
    pub currency: String,
}

/// Payment intent creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: PaymentAmount,
    pub customer_email: String,
    pub description: Option<String>,
    /// Connected account ID for destination charges
    pub destination_account_id: Option<String>,
    /// Commission to retain (in smallest currency unit)
    pub application_fee_amount: Option<i64>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Payment intent result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub client_secret: String,
    pub status: PaymentStatus,
    pub amount: i64,
    pub currency: String,
}

/// Payment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    Succeeded,
    Canceled,
    Failed,
}

/// Merchant account creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMerchantRequest {
    pub email: String,
    pub business_type: String, // "individual" or "company"
    pub country: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Merchant account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantAccount {
    pub id: String,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub requirements: Vec<String>,
    pub onboarding_url: Option<String>,
}

/// Errors from payment provider
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Merchant not found")]
    MerchantNotFound,
    #[error("Payment declined: {0}")]
    Declined(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Trait for payment providers
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Create a payment intent
    async fn create_payment_intent(&self, request: CreatePaymentRequest) -> Result<PaymentIntent, PaymentError>;
    
    /// Retrieve a payment intent status
    async fn get_payment_intent(&self, payment_intent_id: &str) -> Result<PaymentIntent, PaymentError>;
    
    /// Cancel a payment intent
    async fn cancel_payment_intent(&self, payment_intent_id: &str) -> Result<(), PaymentError>;
    
    /// Create a connected account (for landlords)
    async fn create_connected_account(&self, request: CreateMerchantRequest) -> Result<MerchantAccount, PaymentError>;
    
    /// Generate account onboarding link
    async fn create_onboarding_link(&self, account_id: &str, return_url: &str, refresh_url: &str) -> Result<String, PaymentError>;
    
    /// Get account status
    async fn get_account_status(&self, account_id: &str) -> Result<MerchantAccount, PaymentError>;
}

/// Stripe Connect payment provider
pub struct StripeConnectProvider {
    api_key: String,
    client: reqwest::Client,
}

impl StripeConnectProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
    
    fn api_base(&self) -> &str {
        "https://api.stripe.com/v1"
    }
}

#[async_trait]
impl PaymentProvider for StripeConnectProvider {
    async fn create_payment_intent(&self, request: CreatePaymentRequest) -> Result<PaymentIntent, PaymentError> {
        let mut form = vec![
            ("amount".to_string(), request.amount.amount.to_string()),
            ("currency".to_string(), request.amount.currency.to_lowercase()),
            ("receipt_email".to_string(), request.customer_email),
        ];
        
        if let Some(desc) = request.description {
            form.push(("description".to_string(), desc));
        }
        
        // For Connect destination charges
        if let Some(dest) = request.destination_account_id {
            form.push(("transfer_data[destination]".to_string(), dest));
        }
        
        if let Some(fee) = request.application_fee_amount {
            form.push(("application_fee_amount".to_string(), fee.to_string()));
        }
        
        // Add metadata
        for (key, value) in request.metadata {
            form.push((format!("metadata[{}]", key), value));
        }
        
        let response = self.client
            .post(format!("{}/payment_intents", self.api_base()))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await
                .unwrap_or(serde_json::json!({"error": {"message": "Unknown error"}}));
            return Err(PaymentError::ApiError(
                error["error"]["message"].as_str().unwrap_or("Unknown error").to_string()
            ));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        Ok(PaymentIntent {
            id: json["id"].as_str().unwrap_or("").to_string(),
            client_secret: json["client_secret"].as_str().unwrap_or("").to_string(),
            status: parse_payment_status(json["status"].as_str().unwrap_or("")),
            amount: json["amount"].as_i64().unwrap_or(0),
            currency: json["currency"].as_str().unwrap_or("usd").to_string(),
        })
    }
    
    async fn get_payment_intent(&self, payment_intent_id: &str) -> Result<PaymentIntent, PaymentError> {
        let response = self.client
            .get(format!("{}/payment_intents/{}", self.api_base(), payment_intent_id))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PaymentError::ApiError("Failed to retrieve payment intent".to_string()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        Ok(PaymentIntent {
            id: json["id"].as_str().unwrap_or("").to_string(),
            client_secret: json["client_secret"].as_str().unwrap_or("").to_string(),
            status: parse_payment_status(json["status"].as_str().unwrap_or("")),
            amount: json["amount"].as_i64().unwrap_or(0),
            currency: json["currency"].as_str().unwrap_or("usd").to_string(),
        })
    }
    
    async fn cancel_payment_intent(&self, payment_intent_id: &str) -> Result<(), PaymentError> {
        let response = self.client
            .post(format!("{}/payment_intents/{}/cancel", self.api_base(), payment_intent_id))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PaymentError::ApiError("Failed to cancel payment intent".to_string()));
        }
        
        Ok(())
    }
    
    async fn create_connected_account(&self, request: CreateMerchantRequest) -> Result<MerchantAccount, PaymentError> {
        let mut form = vec![
            ("type".to_string(), "express".to_string()),
            ("email".to_string(), request.email),
            ("country".to_string(), request.country),
            ("business_type".to_string(), request.business_type),
            ("capabilities[card_payments][requested]".to_string(), "true".to_string()),
            ("capabilities[transfers][requested]".to_string(), "true".to_string()),
        ];
        
        for (key, value) in request.metadata {
            form.push((format!("metadata[{}]", key), value));
        }
        
        let response = self.client
            .post(format!("{}/accounts", self.api_base()))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error: serde_json::Value = response.json().await
                .unwrap_or(serde_json::json!({"error": {"message": "Unknown error"}}));
            return Err(PaymentError::ApiError(
                error["error"]["message"].as_str().unwrap_or("Unknown error").to_string()
            ));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        Ok(MerchantAccount {
            id: json["id"].as_str().unwrap_or("").to_string(),
            charges_enabled: json["charges_enabled"].as_bool().unwrap_or(false),
            payouts_enabled: json["payouts_enabled"].as_bool().unwrap_or(false),
            requirements: extract_requirements(&json),
            onboarding_url: None,
        })
    }
    
    async fn create_onboarding_link(&self, account_id: &str, return_url: &str, refresh_url: &str) -> Result<String, PaymentError> {
        let form = vec![
            ("account", account_id),
            ("type", "account_onboarding"),
            ("return_url", return_url),
            ("refresh_url", refresh_url),
        ];
        
        let response = self.client
            .post(format!("{}/account_links", self.api_base()))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(PaymentError::ApiError("Failed to create onboarding link".to_string()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        Ok(json["url"].as_str().unwrap_or("").to_string())
    }
    
    async fn get_account_status(&self, account_id: &str) -> Result<MerchantAccount, PaymentError> {
        let response = self.client
            .get(format!("{}/accounts/{}", self.api_base(), account_id))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 404 {
            return Err(PaymentError::MerchantNotFound);
        }
        
        if !response.status().is_success() {
            return Err(PaymentError::ApiError("Failed to get account status".to_string()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;
        
        Ok(MerchantAccount {
            id: json["id"].as_str().unwrap_or("").to_string(),
            charges_enabled: json["charges_enabled"].as_bool().unwrap_or(false),
            payouts_enabled: json["payouts_enabled"].as_bool().unwrap_or(false),
            requirements: extract_requirements(&json),
            onboarding_url: None,
        })
    }
}

fn parse_payment_status(status: &str) -> PaymentStatus {
    match status {
        "requires_payment_method" => PaymentStatus::RequiresPaymentMethod,
        "requires_confirmation" => PaymentStatus::RequiresConfirmation,
        "requires_action" => PaymentStatus::RequiresAction,
        "processing" => PaymentStatus::Processing,
        "succeeded" => PaymentStatus::Succeeded,
        "canceled" => PaymentStatus::Canceled,
        _ => PaymentStatus::Failed,
    }
}

fn extract_requirements(json: &serde_json::Value) -> Vec<String> {
    json["requirements"]["currently_due"]
        .as_array()
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect())
        .unwrap_or_default()
}

// ============================================================================
// ActionCollectPayment Node Handler
// ============================================================================

use core_models::NodeDef;
use crate::context::ExecutionContext;
use crate::NodeEngineError;
use crate::nodes::NodeHandler;
use std::collections::HashMap;
use serde_json::{json, Value};

/// Action Collect Payment - workflow node for collecting payments
pub struct ActionCollectPaymentHandler {
    provider: std::sync::Arc<dyn PaymentProvider>,
}

impl ActionCollectPaymentHandler {
    pub fn new(provider: std::sync::Arc<dyn PaymentProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl NodeHandler for ActionCollectPaymentHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get payment details from config or inputs
        let amount = node.config.get("amount")
            .or(inputs.get("amount"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing amount".into()))?;
        
        let currency = node.config.get("currency")
            .or(inputs.get("currency"))
            .and_then(|v| v.as_str())
            .unwrap_or("USD")
            .to_string();
        
        let email = node.config.get("email")
            .or(inputs.get("customer_email"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing customer email".into()))?;
        
        let description = node.config.get("description")
            .or(inputs.get("description"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Optional destination account for Connect
        let destination = node.config.get("destination_account")
            .or(inputs.get("merchant_stripe_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Commission handling
        let commission_percent = node.config.get("commission_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);
        
        let commission_amount = if destination.is_some() {
            Some((amount * commission_percent / 100.0 * 100.0) as i64) // Convert to cents
        } else {
            None
        };
        
        // Build metadata
        let mut metadata = HashMap::new();
        if let Some(record) = inputs.get("record") {
            if let Some(id) = record.get("id").and_then(|v| v.as_str()) {
                metadata.insert("record_id".to_string(), id.to_string());
            }
        }
        metadata.insert("workflow_execution_id".to_string(), context.request_id.to_string());
        
        // Create payment intent
        let request = CreatePaymentRequest {
            amount: PaymentAmount {
                amount: (amount * 100.0) as i64, // Convert to cents
                currency,
            },
            customer_email: email.to_string(),
            description,
            destination_account_id: destination,
            application_fee_amount: commission_amount,
            metadata,
        };
        
        let result = self.provider.create_payment_intent(request).await
            .map_err(|e| NodeEngineError::ExecutionFailed(e.to_string()))?;
        
        Ok(json!({
            "success": true,
            "payment_intent_id": result.id,
            "client_secret": result.client_secret,
            "status": format!("{:?}", result.status),
            "amount": result.amount,
            "currency": result.currency,
            "commission_amount": commission_amount,
        }))
    }
}
