//! Audit Trail API Routes
//!
//! Provides endpoints for querying audit logs.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::state::AppState;
use crate::middleware::tenant::ResolvedTenant;

/// Audit log entry returned by API
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for audit logs
#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub action: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List audit logs for tenant
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100).min(500);
    let offset = query.offset.unwrap_or(0);
    
    let sql = r#"
        SELECT 
            id, tenant_id, user_id, action, resource_type, resource_id,
            changes, ip_address, user_agent, created_at
        FROM audit_logs
        WHERE tenant_id = $1
            AND ($2::text IS NULL OR resource_type = $2)
            AND ($3::text IS NULL OR resource_id = $3)
            AND ($4::text IS NULL OR action = $4)
            AND ($5::text IS NULL OR user_id = $5)
        ORDER BY created_at DESC
        LIMIT $6 OFFSET $7
    "#;
    
    let rows = sqlx::query(sql)
        .bind(tenant.id)
        .bind(&query.resource_type)
        .bind(&query.resource_id)
        .bind(&query.action)
        .bind(&query.user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await;
    
    match rows {
        Ok(results) => {
            let logs: Vec<AuditLogResponse> = results.iter().map(|row| {
                AuditLogResponse {
                    id: row.get("id"),
                    tenant_id: row.get("tenant_id"),
                    user_id: row.try_get("user_id").ok(),
                    action: row.get("action"),
                    resource_type: row.get("resource_type"),
                    resource_id: row.try_get("resource_id").ok(),
                    changes: row.try_get("changes").ok(),
                    ip_address: row.try_get("ip_address").ok(),
                    user_agent: row.try_get("user_agent").ok(),
                    created_at: row.get("created_at"),
                }
            }).collect();
            Json(logs).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to query audit logs: {}", e)).into_response()
        }
    }
}

/// Get audit logs for a specific entity
pub async fn get_entity_audit_logs(
    State(state): State<Arc<AppState>>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100).min(500);
    let offset = query.offset.unwrap_or(0);
    
    let sql = r#"
        SELECT 
            id, tenant_id, user_id, action, resource_type, resource_id,
            changes, ip_address, user_agent, created_at
        FROM audit_logs
        WHERE tenant_id = $1
            AND resource_type = $2
            AND resource_id = $3
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
    "#;
    
    let rows = sqlx::query(sql)
        .bind(tenant.id)
        .bind(&entity_type)
        .bind(entity_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await;
    
    match rows {
        Ok(results) => {
            let logs: Vec<AuditLogResponse> = results.iter().map(|row| {
                AuditLogResponse {
                    id: row.get("id"),
                    tenant_id: row.get("tenant_id"),
                    user_id: row.try_get("user_id").ok(),
                    action: row.get("action"),
                    resource_type: row.get("resource_type"),
                    resource_id: row.try_get("resource_id").ok(),
                    changes: row.try_get("changes").ok(),
                    ip_address: row.try_get("ip_address").ok(),
                    user_agent: row.try_get("user_agent").ok(),
                    created_at: row.get("created_at"),
                }
            }).collect();
            Json(logs).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to query audit logs: {}", e)).into_response()
        }
    }
}

/// Create audit routes
pub fn audit_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_audit_logs))
        .route("/:entity_type/:entity_id", get(get_entity_audit_logs))
}
