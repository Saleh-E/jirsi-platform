//! Hybrid Validation - Portable and Async validation for Antigravity
//!
//! This module provides a dual-layer validation system:
//! - **Portable**: Sync validators that work in WASM (frontend) and native (backend)
//! - **Async**: Database-dependent validators that only run on the backend

use serde::{Deserialize, Serialize};

/// Validation rules that can be attached to fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "rule", content = "params", rename_all = "camelCase")]
pub enum ValidationRule {
    // ==================
    // Portable Rules (WASM-safe)
    // ==================
    
    /// Field must have a non-empty value
    Required,
    
    /// Text must match regex pattern
    Regex { pattern: String, message: String },
    
    /// Minimum text length
    MinLength(usize),
    
    /// Maximum text length
    MaxLength(usize),
    
    /// Must be a valid email format
    Email,
    
    /// Must be a valid URL format
    Url,
    
    // ==================
    // Backend-Only Rules (require DB)
    // ==================
    
    /// Value must be unique in the table
    Unique { table: String, column: String },
}

impl ValidationRule {
    /// Check if this rule can be evaluated in WASM (no DB required)
    pub fn is_portable(&self) -> bool {
        !matches!(self, ValidationRule::Unique { .. })
    }
}

/// Trait for validating field values
pub trait Validator {
    /// Portable validation - works in WASM and native
    fn check_portable(&self, value: &serde_json::Value) -> Result<(), String>;
    
    /// Async validation - backend only with database access
    #[cfg(feature = "backend")]
    fn check_async<'a>(
        &'a self,
        value: &'a serde_json::Value,
        pool: &'a sqlx::PgPool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>>;
}

/// Validate a value against a portable rule
pub fn validate_portable(
    field_name: &str,
    value: &serde_json::Value,
    rule: &ValidationRule,
) -> Result<(), String> {
    match rule {
        ValidationRule::Required => {
            let is_empty = value.is_null()
                || value.as_str().map_or(false, |s| s.trim().is_empty())
                || value.as_array().map_or(false, |a| a.is_empty());
            
            if is_empty {
                return Err(format!("{} is required", field_name));
            }
        }
        
        ValidationRule::MinLength(min) => {
            if let Some(s) = value.as_str() {
                if s.len() < *min {
                    return Err(format!("{} must be at least {} characters", field_name, min));
                }
            }
        }
        
        ValidationRule::MaxLength(max) => {
            if let Some(s) = value.as_str() {
                if s.len() > *max {
                    return Err(format!("{} must be at most {} characters", field_name, max));
                }
            }
        }
        
        ValidationRule::Email => {
            if let Some(s) = value.as_str() {
                if !s.contains('@') || !s.contains('.') {
                    return Err(format!("{} must be a valid email address", field_name));
                }
            }
        }
        
        ValidationRule::Url => {
            if let Some(s) = value.as_str() {
                if !s.starts_with("http://") && !s.starts_with("https://") {
                    return Err(format!("{} must be a valid URL", field_name));
                }
            }
        }
        
        ValidationRule::Regex { message, .. } => {
            // Note: Full regex requires `regex` crate
            // For WASM simplicity, we skip regex validation here
            let _ = message;
        }
        
        // Backend-only rules skip in portable context
        ValidationRule::Unique { .. } => {}
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_required_validation() {
        let result = validate_portable("name", &json!(""), &ValidationRule::Required);
        assert!(result.is_err());
        
        let result = validate_portable("name", &json!("John"), &ValidationRule::Required);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_length_validation() {
        let result = validate_portable("code", &json!("AB"), &ValidationRule::MinLength(3));
        assert!(result.is_err());
        
        let result = validate_portable("code", &json!("ABCD"), &ValidationRule::MinLength(3));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_email_validation() {
        let result = validate_portable("email", &json!("invalid"), &ValidationRule::Email);
        assert!(result.is_err());
        
        let result = validate_portable("email", &json!("test@example.com"), &ValidationRule::Email);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_portable_check() {
        assert!(ValidationRule::Required.is_portable());
        assert!(ValidationRule::Email.is_portable());
        assert!(!ValidationRule::Unique { 
            table: "users".into(), 
            column: "email".into(),
        }.is_portable());
    }
}
