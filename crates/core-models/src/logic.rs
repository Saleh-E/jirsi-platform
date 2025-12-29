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
    // Data Logic
    // ==================
    
    /// Field equals a specific value
    Equals { field: String, value: Value },
    
    /// Field does not equal a value
    NotEquals { field: String, value: Value },
    
    /// Field is empty/null/missing
    Empty { field: String },
    
    /// Field contains substring (for text) or element (for arrays)
    Contains { field: String, value: Value },
    
    /// Field value is greater than
    Gt { field: String, value: f64 },
    
    /// Field value is less than
    Lt { field: String, value: f64 },
    
    // ==================
    // Context Logic
    // ==================
    
    /// User has a specific role
    HasRole { role: String },
    
    /// Current user is the record owner
    IsOwner,
    
    /// Feature flag is enabled
    FeatureEnabled { flag: String },
    
    /// Device type check for adaptive UI ("mobile" | "desktop")
    DeviceType { device: String },
    
    // ==================
    // Combinators
    // ==================
    
    /// All conditions must be true (AND)
    And(Vec<LogicOp>),
    
    /// Any condition must be true (OR)
    Or(Vec<LogicOp>),
    
    /// Negates the inner condition (NOT)
    Not(Box<LogicOp>),
}

impl Default for LogicOp {
    fn default() -> Self {
        Self::Always
    }
}

/// The context passed from Frontend or Backend for evaluation
#[derive(Debug, Clone)]
pub struct EvalContext<'a> {
    /// User's assigned roles (e.g., ["admin", "sales_rep"])
    pub user_roles: &'a [String],
    
    /// Current user's ID (for IsOwner check)
    pub user_id: Option<&'a str>,
    
    /// Current record data as field -> value map
    pub record_data: &'a HashMap<String, Value>,
    
    /// Enabled feature flags
    pub feature_flags: &'a [String],
    
    /// Current device type ("mobile", "tablet", "desktop")
    pub device_type: &'a str,
    
    /// Owner field value from record (for IsOwner comparison)
    pub record_owner_id: Option<&'a str>,
}

// Static empty data for default context
static EMPTY_STRINGS: &[String] = &[];
use std::sync::LazyLock;
static EMPTY_HASHMAP: LazyLock<HashMap<String, Value>> = LazyLock::new(HashMap::new);

impl<'a> EvalContext<'a> {
    /// Create a new evaluation context with all defaults
    pub fn new() -> EvalContext<'static> {
        EvalContext {
            user_roles: EMPTY_STRINGS,
            user_id: None,
            record_data: &*EMPTY_HASHMAP,
            feature_flags: EMPTY_STRINGS,
            device_type: "desktop",
            record_owner_id: None,
        }
    }
    
    /// Create a context with specific record data
    pub fn with_data(record_data: &'a HashMap<String, Value>) -> Self {
        Self {
            user_roles: EMPTY_STRINGS,
            user_id: None,
            record_data,
            feature_flags: EMPTY_STRINGS,
            device_type: "desktop",
            record_owner_id: None,
        }
    }
}

impl LogicOp {
    /// Evaluate this logic operation against the given context
    /// 
    /// This is the core evaluator - completely WASM-compatible, no async.
    pub fn evaluate(&self, ctx: &EvalContext) -> bool {
        match self {
            // Primitives
            LogicOp::Always => true,
            LogicOp::Never => false,
            
            // Data Logic
            LogicOp::Equals { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| v == value)
            }
            LogicOp::NotEquals { field, value } => {
                ctx.record_data.get(field).map_or(true, |v| v != value)
            }
            LogicOp::Empty { field } => {
                ctx.record_data.get(field).map_or(true, |v| v.is_null())
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
            LogicOp::Gt { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| {
                    v.as_f64().map_or(false, |n| n > *value)
                })
            }
            LogicOp::Lt { field, value } => {
                ctx.record_data.get(field).map_or(false, |v| {
                    v.as_f64().map_or(false, |n| n < *value)
                })
            }
            
            // Context Logic
            LogicOp::HasRole { role } => ctx.user_roles.contains(role),
            LogicOp::IsOwner => {
                match (ctx.user_id, ctx.record_owner_id) {
                    (Some(user), Some(owner)) => user == owner,
                    _ => false,
                }
            }
            LogicOp::FeatureEnabled { flag } => ctx.feature_flags.contains(flag),
            LogicOp::DeviceType { device } => ctx.device_type == device,
            
            // Combinators
            LogicOp::And(ops) => ops.iter().all(|op| op.evaluate(ctx)),
            LogicOp::Or(ops) => ops.iter().any(|op| op.evaluate(ctx)),
            LogicOp::Not(op) => !op.evaluate(ctx),
        }
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
        let ctx = EvalContext { user_roles: &roles, ..base };
        
        assert!(LogicOp::HasRole { role: "admin".into() }.evaluate(&ctx));
        assert!(!LogicOp::HasRole { role: "superuser".into() }.evaluate(&ctx));
    }
    
    #[test]
    fn test_field_equals() {
        let mut data = HashMap::new();
        data.insert("status".to_string(), json!("active"));
        
        let ctx = EvalContext::with_data(&data);
        
        assert!(LogicOp::Equals { 
            field: "status".into(), 
            value: json!("active") 
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
        
        let op = LogicOp::And(vec![
            LogicOp::HasRole { role: "admin".into() },
            LogicOp::Equals { field: "status".into(), value: json!("draft") },
        ]);
        assert!(op.evaluate(&ctx));
    }
    
    #[test]
    fn test_device_type() {
        let base = EvalContext::new();
        let ctx = EvalContext { device_type: "mobile", ..base };
        
        assert!(LogicOp::DeviceType { device: "mobile".into() }.evaluate(&ctx));
        assert!(!LogicOp::DeviceType { device: "desktop".into() }.evaluate(&ctx));
    }
}
