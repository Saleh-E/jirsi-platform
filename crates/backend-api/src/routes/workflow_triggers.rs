//! Workflow Trigger Routes
//!
//! Endpoints for triggering workflows from external sources.
//! Supports webhook triggers with HMAC-SHA256 signature verification.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Header name for webhook signature
const SIGNATURE_HEADER: &str = "X-Webhook-Signature";
const TIMESTAMP_HEADER: &str = "X-Webhook-Timestamp";

/// Webhook trigger request
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookTriggerPayload {
    /// Optional event type for filtering
    pub event_type: Option<String>,
    /// Payload data to pass to workflow
    pub data: serde_json::Value,
}

/// Webhook trigger response
#[derive(Debug, Serialize)]
pub struct WebhookTriggerResponse {
    pub success: bool,
    pub execution_id: Option<Uuid>,
    pub message: String,
}

/// Create workflow trigger routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Invoke a workflow via webhook trigger
        .route(
            "/api/v1/workflows/triggers/:trigger_id/invoke",
            post(invoke_workflow_trigger),
        )
        // Create/manage webhook secrets
        .route(
            "/api/v1/workflows/:workflow_id/webhook-secret",
            post(generate_webhook_secret),
        )
}

/// POST /api/v1/workflows/triggers/:trigger_id/invoke
/// 
/// Invokes a workflow via its webhook trigger.
/// Requires HMAC-SHA256 signature verification.
async fn invoke_workflow_trigger(
    State(state): State<Arc<AppState>>,
    Path(trigger_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    info!(trigger_id = %trigger_id, "Webhook trigger invocation received");

    // 1. Look up the trigger and get its secret
    let trigger_result = sqlx::query_as::<_, (Uuid, Uuid, String, bool, Option<String>)>(
        r#"
        SELECT t.id, t.graph_id, g.tenant_id::text, t.is_active, t.webhook_secret
        FROM workflow_triggers t
        JOIN workflow_graphs g ON t.graph_id = g.id
        WHERE t.id = $1 AND t.trigger_type = 'webhook'
        "#
    )
    .bind(trigger_id)
    .fetch_optional(&state.pool)
    .await;

    let (trigger_id, graph_id, tenant_id_str, is_active, webhook_secret) = match trigger_result {
        Ok(Some(row)) => row,
        Ok(None) => {
            warn!(trigger_id = %trigger_id, "Webhook trigger not found");
            return (
                StatusCode::NOT_FOUND,
                Json(WebhookTriggerResponse {
                    success: false,
                    execution_id: None,
                    message: "Trigger not found".to_string(),
                }),
            );
        }
        Err(e) => {
            error!(error = %e, "Database error looking up trigger");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WebhookTriggerResponse {
                    success: false,
                    execution_id: None,
                    message: "Internal error".to_string(),
                }),
            );
        }
    };

    // 2. Check if trigger is active
    if !is_active {
        warn!(trigger_id = %trigger_id, "Webhook trigger is disabled");
        return (
            StatusCode::FORBIDDEN,
            Json(WebhookTriggerResponse {
                success: false,
                execution_id: None,
                message: "Trigger is disabled".to_string(),
            }),
        );
    }

    // 3. Verify HMAC signature
    let secret = match webhook_secret {
        Some(s) => s,
        None => {
            warn!(trigger_id = %trigger_id, "Webhook trigger has no secret configured");
            return (
                StatusCode::FORBIDDEN,
                Json(WebhookTriggerResponse {
                    success: false,
                    execution_id: None,
                    message: "Webhook secret not configured".to_string(),
                }),
            );
        }
    };

    let signature = headers
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok());

    if !verify_signature(&body, &secret, signature) {
        warn!(trigger_id = %trigger_id, "Invalid webhook signature");
        return (
            StatusCode::UNAUTHORIZED,
            Json(WebhookTriggerResponse {
                success: false,
                execution_id: None,
                message: "Invalid signature".to_string(),
            }),
        );
    }

    // 4. Parse the payload
    let payload: WebhookTriggerPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            warn!(trigger_id = %trigger_id, error = %e, "Invalid webhook payload");
            return (
                StatusCode::BAD_REQUEST,
                Json(WebhookTriggerResponse {
                    success: false,
                    execution_id: None,
                    message: format!("Invalid payload: {}", e),
                }),
            );
        }
    };

    // 5. Parse tenant ID
    let tenant_id = match Uuid::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!(tenant_id = %tenant_id_str, "Invalid tenant ID in database");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WebhookTriggerResponse {
                    success: false,
                    execution_id: None,
                    message: "Internal error".to_string(),
                }),
            );
        }
    };

    // 6. Queue workflow execution
    let execution_id = Uuid::new_v4();
    let trigger_data = serde_json::json!({
        "source": "webhook",
        "trigger_id": trigger_id,
        "event_type": payload.event_type,
        "data": payload.data,
        "received_at": chrono::Utc::now().to_rfc3339(),
    });

    let insert_result = sqlx::query(
        r#"
        INSERT INTO workflow_executions (id, tenant_id, graph_id, trigger_id, trigger_data, status, created_at)
        VALUES ($1, $2, $3, $4, $5, 'pending', NOW())
        "#
    )
    .bind(execution_id)
    .bind(tenant_id)
    .bind(graph_id)
    .bind(trigger_id)
    .bind(&trigger_data)
    .execute(&state.pool)
    .await;

    match insert_result {
        Ok(_) => {
            info!(
                execution_id = %execution_id,
                graph_id = %graph_id,
                trigger_id = %trigger_id,
                "Workflow execution queued from webhook"
            );
            (
                StatusCode::ACCEPTED,
                Json(WebhookTriggerResponse {
                    success: true,
                    execution_id: Some(execution_id),
                    message: "Workflow execution queued".to_string(),
                }),
            )
        }
        Err(e) => {
            error!(error = %e, "Failed to queue workflow execution");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WebhookTriggerResponse {
                    success: false,
                    execution_id: None,
                    message: "Failed to queue execution".to_string(),
                }),
            )
        }
    }
}

/// POST /api/v1/workflows/:workflow_id/webhook-secret
/// 
/// Generates a new webhook secret for a workflow trigger
async fn generate_webhook_secret(
    State(state): State<Arc<AppState>>,
    Path(workflow_id): Path<Uuid>,
) -> impl IntoResponse {
    // Generate a secure random secret
    let secret = generate_secure_secret();

    // Update the trigger with the new secret
    let result = sqlx::query(
        r#"
        UPDATE workflow_triggers 
        SET webhook_secret = $1, updated_at = NOW()
        WHERE graph_id = $2 AND trigger_type = 'webhook'
        RETURNING id
        "#
    )
    .bind(&secret)
    .bind(workflow_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            info!(workflow_id = %workflow_id, "Webhook secret generated");
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "secret": secret,
                    "message": "Store this secret securely - it will not be shown again"
                })),
            )
        }
        Ok(_) => {
            warn!(workflow_id = %workflow_id, "No webhook trigger found for workflow");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "message": "No webhook trigger found for this workflow"
                })),
            )
        }
        Err(e) => {
            error!(error = %e, "Failed to update webhook secret");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to generate secret"
                })),
            )
        }
    }
}

/// Verify HMAC-SHA256 signature
/// 
/// Signature format: sha256=<hex-encoded-signature>
fn verify_signature(body: &[u8], secret: &str, signature: Option<&str>) -> bool {
    let sig = match signature {
        Some(s) => s,
        None => return false,
    };

    // Expect format: sha256=<hex>
    let expected_prefix = "sha256=";
    if !sig.starts_with(expected_prefix) {
        return false;
    }

    let provided_sig = &sig[expected_prefix.len()..];
    
    // Compute expected signature
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    
    // Convert to hex for comparison
    let expected_sig = hex::encode(mac.finalize().into_bytes());
    
    // Constant-time comparison to prevent timing attacks
    constant_time_eq(provided_sig.as_bytes(), expected_sig.as_bytes())
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Generate a cryptographically secure secret
fn generate_secure_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}
