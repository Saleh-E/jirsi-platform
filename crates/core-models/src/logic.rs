//! Logic Engine - Recursive context-aware evaluator for Antigravity
//!
//! This module provides a portable expression evaluator that works identically
//! in both WASM (frontend) and native (backend) contexts. It powers:
//! - Conditional field visibility (`visible_if`)
//! - Dynamic read-only states (`readonly_if`)
//! - Role-based access control
//! - Feature flag gating
//! - Adaptive UI based on device type

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Logic operations for conditional evaluation
/// 
/// Uses tagged serde format for clean JSON representation:
/// ```json
/// { "op": "and", "args": [{ "op": "hasRole", "args": { "role": "admin" } }] }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "op", content = "args")]
pub enum LogicOp {
    // ==================
    // Primitives
    // ==================
    
    /// Always evaluates to true (default for visibility)
    Always,
    
    /// Always evaluates to false (default for readonly)
    Never,
    
    // ==================
    // Field Comparisons
    // ==================
    
    /// Field equals a specific value
    Equals { field: String, value: Value },
    
    /// Field does not equal a value
    NotEquals { field: String, value: Value },
    
    /// Field is empty/null/missing
    Empty { field: String },
    
    /// Field is not empty
    NotEmpty { field: String },
    
    /// Field value is greater than
    GreaterThan { field: String, value: Value },
    
    /// Field value is less than
    LessThan { field: String, value: Value },
    
    /// Field contains substring (for text) or element (for arrays)
    Contains { field: String, value: Value },
    
    /// Field matches regex pattern
    Matches { field: String, pattern: String },
    
    // ==================
    // Access Control
    // ==================
    
    /// User has a specific role
    HasRole { role: String },
    
    /// User has any of the specified roles
    HasAnyRole { roles: Vec<String> },
    
    /// User has all of the specified roles
    HasAllRoles { roles: Vec<String> },
    
    /// Current user is the record owner
    IsOwner,
    
    /// Current user is in a specific team
    InTeam { team: String },
    
    // ==================
    // Feature & Environment
    // ==================
    
    /// Feature flag is enabled
    FeatureEnabled { flag: String },
    
    /// Device type check for adaptive UI (mobile, tablet, desktop)
    DeviceType { device: String },
    
    /// Tenant has specific plan/tier
    TenantPlan { plan: String },
    
    // ==================
    // Combinators
    // ==================
    
    /// All conditions must be true (AND)
    And(Vec<LogicOp>),
    
    /// Any condition must be true (OR)
    Or(Vec<LogicOp>),
    
    /// Negates the inner condition (NOT)
    Not(Box<LogicOp>),
    
    /// If-then-else conditional
    IfThenElse {
        condition: Box<LogicOp>,
        then_op: Box<LogicOp>,
        else_op: Box<LogicOp>,
    },
}

impl Default for LogicOp {
    fn default() -> Self {
        Self::Always
    }
}

/// Evaluation context - all data needed to evaluate LogicOp expressions
/// 
/// This struct is passed to the evaluator and contains:
/// - User identity and roles
/// - Current record data
/// - Feature flags
/// - Device/environment info
#[derive(Debug, Clone)]
pub struct EvalContext<'a> {
    /// User's assigned roles (e.g., ["admin", "sales_rep"])
    pub user_roles: &'a [String],
    
    /// Current user's ID (for IsOwner check)
    pub user_id: Option<&'a str>,
    
    /// User's team memberships
    pub user_teams: &'a [String],
    
    /// Current record data as field -> value map
    pub record_data: &'a HashMap<String, Value>,
    
    /// Owner field value from record (for IsOwner comparison)
    pub record_owner_id: Option<&'a str>,
    
    /// Enabled feature flags
    pub feature_flags: &'a [String],
    
    /// Current device type ("mobile", "tablet", "desktop")
    pub device_type: &'a str,
    
    /// Tenant's plan/tier
    pub tenant_plan: Option<&'a str>,
}

// Static empty data for default context
static EMPTY_STRINGS: &[String] = &[];
use std::sync::LazyLock;
static EMPTY_HASHMAP: LazyLock<HashMap<String, Value>> = LazyLock::new(HashMap::new);

impl<'a> EvalContext<'a> {
    /// Create a new evaluation context with all defaults
    /// 
    /// This is the preferred way to create a context, as it avoids
    /// lifetime issues with Default trait.
    pub fn new() -> EvalContext<'static> {
        EvalContext {
            user_roles: EMPTY_STRINGS,
            user_id: None,
            user_teams: EMPTY_STRINGS,
            record_data: &*EMPTY_HASHMAP,
            record_owner_id: None,
            feature_flags: EMPTY_STRINGS,
            device_type: "desktop",
            tenant_plan: None,
        }
    }
    
    /// Create a context with specific record data
    pub fn with_data(record_data: &'a HashMap<String, Value>) -> Self {
        Self {
            user_roles: EMPTY_STRINGS,
            user_id: None,
            user_teams: EMPTY_STRINGS,
            record_data,
            record_owner_id: None,
            feature_flags: EMPTY_STRINGS,
            device_type: "desktop",
            tenant_plan: None,
        }
    }
}

impl LogicOp {
    /// Evaluate this logic operation against the given context
    /// 
    /// This is the core evaluator - completely WASM-compatible, no async.
    /// 
    /// # Example
    /// ```ignore
    /// let op = LogicOp::And(vec![
    ///     LogicOp::HasRole { role: "admin".into() },
    ///     LogicOp::Equals { field: "status".into(), value: json!("draft") },
    /// ]);
    /// 
    /// let ctx = EvalContext { user_roles: &["admin".into()], .. };
    /// assert!(op.evaluate(&ctx));
    /// ```
    pub fn evaluate(&self, ctx: &EvalContext) -> bool {
        match self {
            // Primitives
            LogicOp::Always => true,
            LogicOp::Never => false,
            
            // Field comparisons
            LogicOp::Equals { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| v == value)
            }
            LogicOp::NotEquals { field, value } => {
                ctx.record_data.get(field).map_or(true, |v| v != value)
            }
            LogicOp::Empty { field } => {
                ctx.record_data.get(field).map_or(true, |v| {
                    v.is_null() || v.as_str().map_or(false, |s| s.is_empty())
                        || v.as_array().map_or(false, |a| a.is_empty())
                })
            }
            LogicOp::NotEmpty { field } => {
                ctx.record_data.get(field).map_or(false, |v| {
                    !v.is_null() && !v.as_str().map_or(false, |s| s.is_empty())
                        && !v.as_array().map_or(false, |a| a.is_empty())
                })
            }
            LogicOp::GreaterThan { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| {
                    match (v.as_f64(), value.as_f64()) {
                        (Some(a), Some(b)) => a > b,
                        _ => false,
                    }
                })
            }
            LogicOp::LessThan { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| {
                    match (v.as_f64(), value.as_f64()) {
                        (Some(a), Some(b)) => a < b,
                        _ => false,
                    }
                })
            }
            LogicOp::Contains { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| {
                    if let Some(s) = v.as_str() {
                        value.as_str().map_or(false, |needle| s.contains(needle))
                    } else if let Some(arr) = v.as_array() {
                        arr.contains(value)
                    } else {
                        false
                    }
                })
            }
            LogicOp::Matches { field, pattern } => {
                // Note: For WASM compatibility, we use simple contains
                // Full regex would require the `regex` crate
                ctx.record_data.get(field).map_or(false, |v| {
                    v.as_str().map_or(false, |s| s.contains(pattern.as_str()))
                })
            }
            
            // Access control
            LogicOp::HasRole { role } => ctx.user_roles.contains(role),
            LogicOp::HasAnyRole { roles } => {
                roles.iter().any(|r| ctx.user_roles.contains(r))
            }
            LogicOp::HasAllRoles { roles } => {
                roles.iter().all(|r| ctx.user_roles.contains(r))
            }
            LogicOp::IsOwner => {
                match (ctx.user_id, ctx.record_owner_id) {
                    (Some(user), Some(owner)) => user == owner,
                    _ => false,
                }
            }
            LogicOp::InTeam { team } => ctx.user_teams.contains(team),
            
            // Feature & Environment
            LogicOp::FeatureEnabled { flag } => ctx.feature_flags.contains(flag),
            LogicOp::DeviceType { device } => ctx.device_type == device,
            LogicOp::TenantPlan { plan } => {
                ctx.tenant_plan.map_or(false, |p| p == plan)
            }
            
            // Combinators
            LogicOp::And(ops) => ops.iter().all(|op| op.evaluate(ctx)),
            LogicOp::Or(ops) => ops.iter().any(|op| op.evaluate(ctx)),
            LogicOp::Not(op) => !op.evaluate(ctx),
            LogicOp::IfThenElse { condition, then_op, else_op } => {
                if condition.evaluate(ctx) {
                    then_op.evaluate(ctx)
                } else {
                    else_op.evaluate(ctx)
                }
            }
        }
    }
    
    /// Helper: Create an AND combination
    pub fn and(ops: Vec<LogicOp>) -> Self {
        LogicOp::And(ops)
    }
    
    /// Helper: Create an OR combination
    pub fn or(ops: Vec<LogicOp>) -> Self {
        LogicOp::Or(ops)
    }
    
    /// Helper: Negate this operation
    pub fn not(self) -> Self {
        LogicOp::Not(Box::new(self))
    }
    
    /// Helper: Check if user has role
    pub fn has_role(role: impl Into<String>) -> Self {
        LogicOp::HasRole { role: role.into() }
    }
    
    /// Helper: Check if field equals value
    pub fn field_equals(field: impl Into<String>, value: Value) -> Self {
        LogicOp::Equals { field: field.into(), value }
    }
    
    /// Helper: Check device type
    pub fn is_device(device: impl Into<String>) -> Self {
        LogicOp::DeviceType { device: device.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_basic_evaluation() {
        let ctx = EvalContext::new();
        
        assert!(LogicOp::Always.evaluate(&ctx));
        assert!(!LogicOp::Never.evaluate(&ctx));
    }
    
    #[test]
    fn test_role_check() {
        let roles = vec!["admin".to_string(), "sales".to_string()];
        let base = EvalContext::new();
        let ctx = EvalContext {
            user_roles: &roles,
            ..base
        };
        
        assert!(LogicOp::HasRole { role: "admin".into() }.evaluate(&ctx));
        assert!(!LogicOp::HasRole { role: "superuser".into() }.evaluate(&ctx));
    }
    
    #[test]
    fn test_field_equals() {
        let mut data = HashMap::new();
        data.insert("status".to_string(), json!("active"));
        data.insert("count".to_string(), json!(42));
        
        let ctx = EvalContext::with_data(&data);
        
        assert!(LogicOp::Equals { 
            field: "status".into(), 
            value: json!("active") 
        }.evaluate(&ctx));
        
        assert!(!LogicOp::Equals { 
            field: "status".into(), 
            value: json!("inactive") 
        }.evaluate(&ctx));
    }
    
    #[test]
    fn test_combinators() {
        let roles = vec!["admin".to_string()];
        let mut data = HashMap::new();
        data.insert("status".to_string(), json!("draft"));
        
        let base = EvalContext::new();
        let ctx = EvalContext {
            user_roles: &roles,
            record_data: &data,
            ..base
        };
        
        // AND: admin AND status=draft
        let op = LogicOp::And(vec![
            LogicOp::HasRole { role: "admin".into() },
            LogicOp::Equals { field: "status".into(), value: json!("draft") },
        ]);
        assert!(op.evaluate(&ctx));
        
        // NOT
        let not_op = LogicOp::Not(Box::new(LogicOp::Never));
        assert!(not_op.evaluate(&ctx));
    }
    
    #[test]
    fn test_device_type() {
        let base = EvalContext::new();
        let ctx = EvalContext {
            device_type: "mobile",
            ..base
        };
        
        assert!(LogicOp::DeviceType { device: "mobile".into() }.evaluate(&ctx));
        assert!(!LogicOp::DeviceType { device: "desktop".into() }.evaluate(&ctx));
    }
}
