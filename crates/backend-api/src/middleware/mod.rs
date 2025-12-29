//! Middleware modules
//!
//! ## Antigravity Integration
//! Includes LogicOp-based permission middleware for RBAC.

pub mod tenant;
pub mod database;
pub mod rate_limit;
pub mod audit_log;
pub mod permission;

pub use rate_limit::{RateLimiter, RateLimitConfig, SharedRateLimiter, rate_limit_middleware};
pub use audit_log::{AuditLogger, SharedAuditLogger, AuditLogEntry, AuditAction, audit_log_middleware};
pub use permission::{
    AuthenticatedUser, PermissionDef, PermissionContext, PermissionCheckResult,
    check_permission, has_role, is_admin, is_admin_or_manager, can_access,
    get_user_permissions, require_admin, require_manager,
};
