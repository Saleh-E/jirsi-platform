//! Permission Middleware - Antigravity RBAC using LogicOp
//!
//! Provides dynamic, data-driven permission checks using the Logic Engine.
//! Permissions are defined as LogicOp rules in the database, enabling:
//! - Role-based access control (RBAC)
//! - Attribute-based access control (ABAC)
//! - Context-aware permissions (record ownership, field values, etc.)

use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use core_models::logic::{LogicOp, EvalContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// User context available in request extensions (set by auth layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub roles: Vec<String>, // For multi-role support
}

impl AuthenticatedUser {
    /// Get roles as slice for EvalContext
    pub fn role_strings(&self) -> Vec<String> {
        if self.roles.is_empty() {
            vec![self.role.clone()]
        } else {
            self.roles.clone()
        }
    }
}

/// Permission definition - stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDef {
    pub id: Uuid,
    pub name: String,
    pub resource: String,       // e.g., "contact", "deal", "workflow"
    pub action: String,         // e.g., "read", "write", "delete", "execute"
    pub condition: LogicOp,     // The logic rule that must pass
    pub description: Option<String>,
}

/// Result of permission check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub permission_name: String,
    pub reason: Option<String>,
}

/// Permission context for evaluation
#[derive(Debug, Clone, Default)]
pub struct PermissionContext {
    pub record_data: HashMap<String, Value>,
    pub feature_flags: Vec<String>,
    pub device_type: String,
}

impl PermissionContext {
    pub fn new() -> Self {
        Self {
            record_data: HashMap::new(),
            feature_flags: Vec::new(),
            device_type: "desktop".to_string(),
        }
    }
    
    pub fn with_record(mut self, record: HashMap<String, Value>) -> Self {
        self.record_data = record;
        self
    }
}

/// Check if a user has a specific permission
/// 
/// This is the core RBAC function that evaluates LogicOp rules.
/// 
/// # Example
/// ```rust,ignore
/// let permission = PermissionDef {
///     condition: LogicOp::HasRole { role: "admin".to_string() },
///     ..
/// };
/// let allowed = check_permission(&user, &permission, &context);
/// ```
pub fn check_permission(
    user: &AuthenticatedUser,
    permission: &PermissionDef,
    context: &PermissionContext,
) -> PermissionCheckResult {
    let roles = user.role_strings();
    let role_refs: Vec<String> = roles.clone();
    
    // Build EvalContext
    let user_id_str = user.id.to_string();
    let record_owner = context.record_data
        .get("owner_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    
    let eval_ctx = EvalContext {
        user_roles: &role_refs,
        user_id: Some(&user_id_str),
        record_data: &context.record_data,
        feature_flags: &context.feature_flags,
        device_type: &context.device_type,
        record_owner_id: record_owner.as_deref(),
    };
    
    // Evaluate the permission condition
    let allowed = permission.condition.evaluate(&eval_ctx);
    
    PermissionCheckResult {
        allowed,
        permission_name: permission.name.clone(),
        reason: if allowed {
            None
        } else {
            Some(format!("Permission '{}' denied for role '{}'", permission.name, user.role))
        },
    }
}

/// Quick check for common role-based permissions
pub fn has_role(user: &AuthenticatedUser, required_role: &str) -> bool {
    let permission = PermissionDef {
        id: Uuid::nil(),
        name: format!("role:{}", required_role),
        resource: "*".to_string(),
        action: "*".to_string(),
        condition: LogicOp::HasRole { role: required_role.to_string() },
        description: None,
    };
    
    let ctx = PermissionContext::new();
    check_permission(user, &permission, &ctx).allowed
}

/// Check if user is admin
pub fn is_admin(user: &AuthenticatedUser) -> bool {
    has_role(user, "admin")
}

/// Check if user is admin or manager
pub fn is_admin_or_manager(user: &AuthenticatedUser) -> bool {
    has_role(user, "admin") || has_role(user, "manager")
}

/// Check if user can access a specific resource with action
pub fn can_access(
    user: &AuthenticatedUser,
    resource: &str,
    action: &str,
    record: Option<HashMap<String, Value>>,
) -> bool {
    // Default permission rules based on role
    let condition = match (resource, action) {
        // Admin can do anything
        (_, _) if has_role(user, "admin") => return true,
        
        // Managers can do most things
        (_, "read") => LogicOp::Always,
        (_, "write") if has_role(user, "manager") => LogicOp::Always,
        (_, "delete") if has_role(user, "manager") => LogicOp::Always,
        
        // Agents can read and write
        (_, "read") => LogicOp::Always,
        (_, "write") if has_role(user, "agent") => LogicOp::Always,
        
        // Viewers can only read
        (_, "read") if has_role(user, "viewer") => LogicOp::Always,
        
        _ => LogicOp::Never,
    };
    
    let permission = PermissionDef {
        id: Uuid::nil(),
        name: format!("{}:{}", resource, action),
        resource: resource.to_string(),
        action: action.to_string(),
        condition,
        description: None,
    };
    
    let ctx = if let Some(rec) = record {
        PermissionContext::new().with_record(rec)
    } else {
        PermissionContext::new()
    };
    
    check_permission(user, &permission, &ctx).allowed
}

/// Middleware to require a specific permission
/// 
/// Use with axum::middleware::from_fn:
/// ```rust,ignore
/// let app = Router::new()
///     .route("/admin", get(admin_handler))
///     .layer(axum::middleware::from_fn(require_permission("admin_panel", "access")));
/// ```
pub async fn require_admin(
    Extension(user): Extension<AuthenticatedUser>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if is_admin(&user) {
        next.run(request).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Admin access required",
                "code": "PERMISSION_DENIED"
            }))
        ).into_response()
    }
}

/// Middleware to require admin or manager role
pub async fn require_manager(
    Extension(user): Extension<AuthenticatedUser>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if is_admin_or_manager(&user) {
        next.run(request).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Manager or Admin access required",
                "code": "PERMISSION_DENIED"
            }))
        ).into_response()
    }
}

/// Get permissions for a user (for frontend sidebar visibility)
/// 
/// Returns a map of resource:action -> allowed
pub fn get_user_permissions(user: &AuthenticatedUser) -> HashMap<String, bool> {
    let resources = ["contact", "company", "deal", "property", "workflow", "report", "settings"];
    let actions = ["read", "write", "delete"];
    
    let mut permissions = HashMap::new();
    
    for resource in &resources {
        for action in &actions {
            let key = format!("{}:{}", resource, action);
            let allowed = can_access(user, resource, action, None);
            permissions.insert(key, allowed);
        }
    }
    
    // Special permissions
    permissions.insert("admin_panel:access".to_string(), is_admin(user));
    permissions.insert("workflows:execute".to_string(), is_admin_or_manager(user));
    permissions.insert("reports:export".to_string(), is_admin_or_manager(user));
    
    permissions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_user(role: &str) -> AuthenticatedUser {
        AuthenticatedUser {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            role: role.to_string(),
            roles: vec![role.to_string()],
        }
    }
    
    #[test]
    fn test_admin_has_all_permissions() {
        let admin = create_test_user("admin");
        assert!(is_admin(&admin));
        assert!(can_access(&admin, "contact", "delete", None));
        assert!(can_access(&admin, "settings", "write", None));
    }
    
    #[test]
    fn test_viewer_read_only() {
        let viewer = create_test_user("viewer");
        assert!(!is_admin(&viewer));
        assert!(can_access(&viewer, "contact", "read", None));
        // Viewers should not be able to delete (default rules)
    }
    
    #[test]
    fn test_permission_check_with_logic_op() {
        let user = create_test_user("manager");
        let permission = PermissionDef {
            id: Uuid::new_v4(),
            name: "access_reports".to_string(),
            resource: "report".to_string(),
            action: "read".to_string(),
            condition: LogicOp::HasRole { role: "manager".to_string() },
            description: Some("Allow managers to access reports".to_string()),
        };
        
        let ctx = PermissionContext::new();
        let result = check_permission(&user, &permission, &ctx);
        
        assert!(result.allowed);
    }
}
