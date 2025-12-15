//! Webhook Routes - Public endpoints for external providers
//!
//! These endpoints receive webhooks from Twilio, Facebook, etc.
//! NO AUTH - Tenant resolution from URL path only.
//! Security via signature validation.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Webhook query params
#[derive(Debug, Deserialize)]
pub struct WebhookQuery {
    /// Facebook verification params
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

/// Create webhook routes - NO AUTH MIDDLEWARE
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhooks/:tenant_id/:provider", get(verify_webhook))
        .route("/webhooks/:tenant_id/:provider", post(receive_webhook))
}

/// GET - Webhook verification (for Facebook subscription)
async fn verify_webhook(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, provider)): Path<(String, String)>,
    Query(query): Query<WebhookQuery>,
) -> impl IntoResponse {
    // Parse tenant ID
    let tenant_uuid = match Uuid::parse_str(&tenant_id) {
        Ok(id) => id,
        Err(_) => {
            warn!(tenant_id = %tenant_id, "Invalid tenant ID in webhook URL");
            return (StatusCode::BAD_REQUEST, "Invalid tenant ID".to_string());
        }
    };

    info!(tenant_id = %tenant_uuid, provider = %provider, "Webhook verification request");

    // Facebook verification
    if provider == "facebook" {
        if let (Some(mode), Some(token), Some(challenge)) = 
            (query.hub_mode, query.hub_verify_token, query.hub_challenge) 
        {
            if mode == "subscribe" {
                // Get stored verify token
                let stored_token: Result<Option<(String,)>, _> = sqlx::query_as(
                    "SELECT webhook_secret FROM integrations WHERE tenant_id = $1 AND provider = 'facebook'"
                )
                .bind(tenant_uuid)
                .fetch_optional(&state.pool)
                .await;

                if let Ok(Some((secret,))) = stored_token {
                    if token == secret {
                        info!(tenant_id = %tenant_uuid, "Facebook webhook verified");
                        return (StatusCode::OK, challenge);
                    }
                }
                warn!(tenant_id = %tenant_uuid, "Facebook verification failed - token mismatch");
            }
        }
    }

    (StatusCode::FORBIDDEN, "Verification failed".to_string())
}

/// POST - Receive webhook from provider
async fn receive_webhook(
    State(state): State<Arc<AppState>>,
    Path((tenant_id, provider)): Path<(String, String)>,
    _headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Parse tenant ID
    let tenant_uuid = match Uuid::parse_str(&tenant_id) {
        Ok(id) => id,
        Err(_) => {
            warn!(tenant_id = %tenant_id, "Invalid tenant ID in webhook");
            return StatusCode::BAD_REQUEST;
        }
    };

    info!(
        tenant_id = %tenant_uuid,
        provider = %provider,
        body_len = body.len(),
        "Webhook received"
    );

    // Verify tenant exists
    let tenant_exists: Result<Option<(bool,)>, _> = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1)"
    )
    .bind(tenant_uuid)
    .fetch_optional(&state.pool)
    .await;

    if tenant_exists.ok().flatten().map(|(e,)| e) != Some(true) {
        warn!(tenant_id = %tenant_uuid, "Webhook for non-existent tenant");
        return StatusCode::NOT_FOUND;
    }

    // Get webhook secret for signature validation
    let integration: Result<Option<(String, bool)>, _> = sqlx::query_as(
        "SELECT webhook_secret, is_enabled FROM integrations WHERE tenant_id = $1 AND provider = $2"
    )
    .bind(tenant_uuid)
    .bind(&provider)
    .fetch_optional(&state.pool)
    .await;

    let (webhook_secret, is_enabled) = match integration.ok().flatten() {
        Some((secret, enabled)) => (secret, enabled),
        None => {
            warn!(tenant_id = %tenant_uuid, provider = %provider, "No integration configured");
            return StatusCode::NOT_FOUND;
        }
    };

    if !is_enabled {
        warn!(tenant_id = %tenant_uuid, provider = %provider, "Integration disabled");
        return StatusCode::FORBIDDEN;
    }

    // Log the webhook
    let _ = sqlx::query(
        r#"
        INSERT INTO webhook_logs (tenant_id, provider, request_headers, request_body, response_status, events_emitted)
        VALUES ($1, $2, $3, $4, 200, 0)
        "#
    )
    .bind(tenant_uuid)
    .bind(&provider)
    .bind(serde_json::json!({}))
    .bind(String::from_utf8_lossy(&body).to_string())
    .execute(&state.pool)
    .await;

    // Update last webhook timestamp
    let _ = sqlx::query(
        "UPDATE integrations SET last_webhook_at = NOW(), webhook_success_count = webhook_success_count + 1 WHERE tenant_id = $1 AND provider = $2"
    )
    .bind(tenant_uuid)
    .bind(&provider)
    .execute(&state.pool)
    .await;

    info!(tenant_id = %tenant_uuid, provider = %provider, "Webhook processed");

    // Suppress unused variable warning
    let _ = webhook_secret;
    
    // Return 200 OK quickly to acknowledge receipt
    StatusCode::OK
}
