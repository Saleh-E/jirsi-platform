//! Tenant-Aware Audit Logging
//!
//! Comprehensive audit trail for all tenant operations.

use axum::{
    body::Body,
    extract::Request,
    http::Response,
    middleware::Next,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: JsonValue,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Audit action types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    Export,
    Import,
    BulkAction,
    WorkflowTrigger,
    ApiCall,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Create => write!(f, "create"),
            AuditAction::Read => write!(f, "read"),
            AuditAction::Update => write!(f, "update"),
            AuditAction::Delete => write!(f, "delete"),
            AuditAction::Login => write!(f, "login"),
            AuditAction::Logout => write!(f, "logout"),
            AuditAction::Export => write!(f, "export"),
            AuditAction::Import => write!(f, "import"),
            AuditAction::BulkAction => write!(f, "bulk_action"),
            AuditAction::WorkflowTrigger => write!(f, "workflow_trigger"),
            AuditAction::ApiCall => write!(f, "api_call"),
        }
    }
}

/// Shared audit logger
pub type SharedAuditLogger = Arc<AuditLogger>;

/// Audit logger service
pub struct AuditLogger {
    pool: PgPool,
    /// Log read operations (can be noisy)
    log_reads: bool,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            log_reads: false,
        }
    }

    /// Create with read logging enabled
    pub fn with_read_logging(pool: PgPool) -> Self {
        Self {
            pool,
            log_reads: true,
        }
    }

    /// Log an audit event
    pub async fn log(&self, entry: AuditLogEntry) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs 
                (id, tenant_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, created_at)
            VALUES 
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(entry.id)
        .bind(entry.tenant_id)
        .bind(entry.user_id)
        .bind(entry.action)
        .bind(entry.resource_type)
        .bind(entry.resource_id)
        .bind(&entry.details)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log audit entry: {}", e))?;

        Ok(())
    }

    /// Log a simple action
    pub async fn log_action(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: &str,
        resource_id: Option<Uuid>,
        details: JsonValue,
    ) -> Result<(), String> {
        // Skip read logging if disabled
        if matches!(action, AuditAction::Read) && !self.log_reads {
            return Ok(());
        }

        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id,
            details,
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        };

        self.log(entry).await
    }

    /// Query audit logs for a tenant
    pub async fn query_logs(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntry>, String> {
        let entries: Vec<(Uuid, Uuid, Option<Uuid>, String, String, Option<Uuid>, JsonValue, Option<String>, Option<String>, DateTime<Utc>)> = 
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, created_at
                FROM audit_logs
                WHERE tenant_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to query audit logs: {}", e))?;

        Ok(entries.into_iter().map(|row| AuditLogEntry {
            id: row.0,
            tenant_id: row.1,
            user_id: row.2,
            action: row.3,
            resource_type: row.4,
            resource_id: row.5,
            details: row.6,
            ip_address: row.7,
            user_agent: row.8,
            created_at: row.9,
        }).collect())
    }

    /// Query audit logs for a specific resource
    pub async fn query_resource_history(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<AuditLogEntry>, String> {
        let entries: Vec<(Uuid, Uuid, Option<Uuid>, String, String, Option<Uuid>, JsonValue, Option<String>, Option<String>, DateTime<Utc>)> = 
            sqlx::query_as(
                r#"
                SELECT id, tenant_id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, created_at
                FROM audit_logs
                WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3
                ORDER BY created_at DESC
                "#
            )
            .bind(tenant_id)
            .bind(resource_type)
            .bind(resource_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to query resource history: {}", e))?;

        Ok(entries.into_iter().map(|row| AuditLogEntry {
            id: row.0,
            tenant_id: row.1,
            user_id: row.2,
            action: row.3,
            resource_type: row.4,
            resource_id: row.5,
            details: row.6,
            ip_address: row.7,
            user_agent: row.8,
            created_at: row.9,
        }).collect())
    }
}

/// Audit logging middleware
pub async fn audit_log_middleware(
    request: Request,
    next: Next,
) -> Response<Body> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = std::time::Instant::now();

    // Extract tenant and user from extensions
    let tenant_id = request.extensions().get::<Uuid>().copied();
    let user_id = request.extensions().get::<UserId>().map(|u| u.0);
    let audit_logger = request.extensions().get::<SharedAuditLogger>().cloned();

    let response = next.run(request).await;

    // Log after response (non-blocking)
    if let (Some(tenant_id), Some(logger)) = (tenant_id, audit_logger) {
        let status = response.status().as_u16();
        let duration_ms = start.elapsed().as_millis() as u64;
        let action = method_to_action(&method);

        tokio::spawn(async move {
            let details = serde_json::json!({
                "path": path,
                "method": method.as_str(),
                "status": status,
                "duration_ms": duration_ms,
            });

            if let Err(e) = logger.log_action(
                tenant_id,
                user_id,
                action,
                "api",
                None,
                details,
            ).await {
                warn!(error = %e, "Failed to log audit entry");
            }
        });
    }

    response
}

/// User ID wrapper for extension
#[derive(Debug, Clone, Copy)]
pub struct UserId(pub Uuid);

/// Convert HTTP method to audit action
fn method_to_action(method: &axum::http::Method) -> AuditAction {
    match method.as_str() {
        "POST" => AuditAction::Create,
        "GET" => AuditAction::Read,
        "PUT" | "PATCH" => AuditAction::Update,
        "DELETE" => AuditAction::Delete,
        _ => AuditAction::ApiCall,
    }
}
