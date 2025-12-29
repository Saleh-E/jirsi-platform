//! Hybrid Validation - Portable and Async validation for Antigravity
//!
//! This module provides a dual-layer validation system:
//! - **Portable**: Sync validators that work in WASM (frontend) and native (backend)
//! - **Async**: Database-dependent validators that only run on the backend
//!
//! The split allows instant client-side validation while reserving expensive
//! checks (uniqueness, foreign key existence) for server-side.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

// ============================================================================
// VALIDATION ERRORS
// ============================================================================

/// Validation error with field context
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct ValidationError {
    /// Field name that failed validation
    pub field: String,
    
    /// Rule that was violated
    pub rule: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional context (e.g., min length, pattern)
    #[serde(default)]
    pub context: Option<Value>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, rule: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            rule: rule.into(),
            message: message.into(),
            context: None,
        }
    }
    
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }
}

/// Collection of validation errors
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }
    
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
    
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn for_field(&self, field: &str) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.field == field).collect()
    }
}

// ============================================================================
// VALIDATION RULES
// ============================================================================

/// Validation rules that can be attached to fields
/// 
/// Rules are tagged by type for JSON serialization:
/// ```json
/// { "rule": "minLength", "params": 3 }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "rule", content = "params", rename_all = "camelCase")]
pub enum ValidationRule {
    // ==================
    // Portable Rules (WASM-safe)
    // ==================
    
    /// Field must have a non-empty value
    Required,
    
    /// Text must match regex pattern
    Regex { 
        pattern: String, 
        message: String,
    },
    
    /// Minimum text length
    MinLength(usize),
    
    /// Maximum text length
    MaxLength(usize),
    
    /// Text length must be in range
    LengthRange { min: usize, max: usize },
    
    /// Minimum numeric value
    Min(f64),
    
    /// Maximum numeric value
    Max(f64),
    
    /// Numeric value must be in range
    Range { min: f64, max: f64 },
    
    /// Must be a valid email format
    Email,
    
    /// Must be a valid URL format
    Url,
    
    /// Must be a valid phone number format
    Phone,
    
    /// Must be a valid UUID
    Uuid,
    
    /// Must be one of the specified values
    OneOf { values: Vec<Value> },
    
    /// Must not be one of the specified values
    NotOneOf { values: Vec<Value> },
    
    /// Array must have minimum items
    MinItems(usize),
    
    /// Array must have maximum items
    MaxItems(usize),
    
    /// Custom validation with error message
    Custom { 
        validator: String, // Name of custom validator function
        message: String,
    },
    
    // ==================
    // Backend-Only Rules (require DB)
    // ==================
    
    /// Value must be unique in the table
    Unique { 
        table: String, 
        column: String,
        #[serde(default)]
        scope: Option<String>, // Optional: scope uniqueness to tenant
    },
    
    /// Value must exist as a foreign key
    Exists { 
        table: String, 
        column: String,
    },
    
    /// Custom async validator
    AsyncCustom {
        validator: String,
        message: String,
    },
}

impl ValidationRule {
    /// Check if this rule can be evaluated in WASM (no DB required)
    pub fn is_portable(&self) -> bool {
        !matches!(
            self,
            ValidationRule::Unique { .. } 
            | ValidationRule::Exists { .. }
            | ValidationRule::AsyncCustom { .. }
        )
    }
    
    /// Get the rule name for error reporting
    pub fn rule_name(&self) -> &'static str {
        match self {
            ValidationRule::Required => "required",
            ValidationRule::Regex { .. } => "regex",
            ValidationRule::MinLength(_) => "minLength",
            ValidationRule::MaxLength(_) => "maxLength",
            ValidationRule::LengthRange { .. } => "lengthRange",
            ValidationRule::Min(_) => "min",
            ValidationRule::Max(_) => "max",
            ValidationRule::Range { .. } => "range",
            ValidationRule::Email => "email",
            ValidationRule::Url => "url",
            ValidationRule::Phone => "phone",
            ValidationRule::Uuid => "uuid",
            ValidationRule::OneOf { .. } => "oneOf",
            ValidationRule::NotOneOf { .. } => "notOneOf",
            ValidationRule::MinItems(_) => "minItems",
            ValidationRule::MaxItems(_) => "maxItems",
            ValidationRule::Custom { .. } => "custom",
            ValidationRule::Unique { .. } => "unique",
            ValidationRule::Exists { .. } => "exists",
            ValidationRule::AsyncCustom { .. } => "asyncCustom",
        }
    }
}

// ============================================================================
// PORTABLE VALIDATOR (WASM-safe)
// ============================================================================

/// Validate a value against a rule (sync, WASM-compatible)
/// 
/// Returns Ok(()) if valid, Err(message) if invalid
pub fn validate_portable(
    field_name: &str,
    value: &Value,
    rule: &ValidationRule,
) -> Result<(), ValidationError> {
    match rule {
        ValidationRule::Required => {
            let is_empty = value.is_null() 
                || value.as_str().map_or(false, |s| s.trim().is_empty())
                || value.as_array().map_or(false, |a| a.is_empty());
            
            if is_empty {
                return Err(ValidationError::new(
                    field_name,
                    "required",
                    format!("{} is required", field_name),
                ));
            }
        }
        
        ValidationRule::MinLength(min) => {
            if let Some(s) = value.as_str() {
                if s.len() < *min {
                    return Err(ValidationError::new(
                        field_name,
                        "minLength",
                        format!("{} must be at least {} characters", field_name, min),
                    ));
                }
            }
        }
        
        ValidationRule::MaxLength(max) => {
            if let Some(s) = value.as_str() {
                if s.len() > *max {
                    return Err(ValidationError::new(
                        field_name,
                        "maxLength",
                        format!("{} must be at most {} characters", field_name, max),
                    ));
                }
            }
        }
        
        ValidationRule::LengthRange { min, max } => {
            if let Some(s) = value.as_str() {
                let len = s.len();
                if len < *min || len > *max {
                    return Err(ValidationError::new(
                        field_name,
                        "lengthRange",
                        format!("{} must be between {} and {} characters", field_name, min, max),
                    ));
                }
            }
        }
        
        ValidationRule::Min(min) => {
            if let Some(n) = value.as_f64() {
                if n < *min {
                    return Err(ValidationError::new(
                        field_name,
                        "min",
                        format!("{} must be at least {}", field_name, min),
                    ));
                }
            }
        }
        
        ValidationRule::Max(max) => {
            if let Some(n) = value.as_f64() {
                if n > *max {
                    return Err(ValidationError::new(
                        field_name,
                        "max",
                        format!("{} must be at most {}", field_name, max),
                    ));
                }
            }
        }
        
        ValidationRule::Range { min, max } => {
            if let Some(n) = value.as_f64() {
                if n < *min || n > *max {
                    return Err(ValidationError::new(
                        field_name,
                        "range",
                        format!("{} must be between {} and {}", field_name, min, max),
                    ));
                }
            }
        }
        
        ValidationRule::Email => {
            if let Some(s) = value.as_str() {
                // Simple email validation (contains @ and .)
                if !s.contains('@') || !s.contains('.') {
                    return Err(ValidationError::new(
                        field_name,
                        "email",
                        format!("{} must be a valid email address", field_name),
                    ));
                }
            }
        }
        
        ValidationRule::Url => {
            if let Some(s) = value.as_str() {
                if !s.starts_with("http://") && !s.starts_with("https://") {
                    return Err(ValidationError::new(
                        field_name,
                        "url",
                        format!("{} must be a valid URL", field_name),
                    ));
                }
            }
        }
        
        ValidationRule::Phone => {
            if let Some(s) = value.as_str() {
                // Simple phone validation: digits, spaces, +, -, (, )
                let cleaned: String = s.chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if cleaned.len() < 7 || cleaned.len() > 15 {
                    return Err(ValidationError::new(
                        field_name,
                        "phone",
                        format!("{} must be a valid phone number", field_name),
                    ));
                }
            }
        }
        
        ValidationRule::Uuid => {
            if let Some(s) = value.as_str() {
                if uuid::Uuid::parse_str(s).is_err() {
                    return Err(ValidationError::new(
                        field_name,
                        "uuid",
                        format!("{} must be a valid UUID", field_name),
                    ));
                }
            }
        }
        
        ValidationRule::OneOf { values } => {
            if !values.contains(value) {
                return Err(ValidationError::new(
                    field_name,
                    "oneOf",
                    format!("{} must be one of the allowed values", field_name),
                ));
            }
        }
        
        ValidationRule::NotOneOf { values } => {
            if values.contains(value) {
                return Err(ValidationError::new(
                    field_name,
                    "notOneOf",
                    format!("{} contains a forbidden value", field_name),
                ));
            }
        }
        
        ValidationRule::MinItems(min) => {
            if let Some(arr) = value.as_array() {
                if arr.len() < *min {
                    return Err(ValidationError::new(
                        field_name,
                        "minItems",
                        format!("{} must have at least {} items", field_name, min),
                    ));
                }
            }
        }
        
        ValidationRule::MaxItems(max) => {
            if let Some(arr) = value.as_array() {
                if arr.len() > *max {
                    return Err(ValidationError::new(
                        field_name,
                        "maxItems",
                        format!("{} must have at most {} items", field_name, max),
                    ));
                }
            }
        }
        
        ValidationRule::Regex { pattern, message } => {
            if let Some(s) = value.as_str() {
                // Note: Full regex requires `regex` crate
                // For WASM, we use simple contains check as fallback
                // In production, you'd want to add `regex` with wasm support
                if !s.contains(pattern.as_str()) {
                    return Err(ValidationError::new(
                        field_name,
                        "regex",
                        message.clone(),
                    ));
                }
            }
        }
        
        ValidationRule::Custom { message, .. } => {
            // Custom validators are handled by the application layer
            // Here we just return Ok - the actual validation happens elsewhere
            let _ = message; // Suppress unused warning
        }
        
        // Backend-only rules return Ok in portable context
        ValidationRule::Unique { .. } 
        | ValidationRule::Exists { .. } 
        | ValidationRule::AsyncCustom { .. } => {
            // These require database access, skip in portable validation
        }
    }
    
    Ok(())
}

/// Validate a value against multiple rules
pub fn validate_all_portable(
    field_name: &str,
    value: &Value,
    rules: &[ValidationRule],
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    
    for rule in rules.iter().filter(|r| r.is_portable()) {
        if let Err(e) = validate_portable(field_name, value, rule) {
            errors.add(e);
        }
    }
    
    errors
}

// ============================================================================
// ASYNC VALIDATOR (Backend-only)
// ============================================================================

/// Async validation trait for backend-only checks
#[cfg(feature = "backend")]
pub trait AsyncValidator: Send + Sync {
    /// Validate asynchronously (e.g., database uniqueness check)
    fn validate_async<'a>(
        &'a self,
        field_name: &'a str,
        value: &'a Value,
        pool: &'a sqlx::PgPool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ValidationError>> + Send + 'a>>;
}

#[cfg(feature = "backend")]
impl AsyncValidator for ValidationRule {
    fn validate_async<'a>(
        &'a self,
        field_name: &'a str,
        value: &'a Value,
        pool: &'a sqlx::PgPool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ValidationError>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                ValidationRule::Unique { table, column, scope } => {
                    // Build query dynamically
                    let query = if scope.is_some() {
                        format!(
                            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1 AND tenant_id = $2) as exists",
                            table, column
                        )
                    } else {
                        format!(
                            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1) as exists",
                            table, column
                        )
                    };
                    
                    let value_str = value.as_str().unwrap_or_default();
                    
                    let exists: (bool,) = sqlx::query_as(&query)
                        .bind(value_str)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| ValidationError::new(
                            field_name,
                            "unique",
                            format!("Database error: {}", e),
                        ))?;
                    
                    if exists.0 {
                        return Err(ValidationError::new(
                            field_name,
                            "unique",
                            format!("{} must be unique", field_name),
                        ));
                    }
                }
                
                ValidationRule::Exists { table, column } => {
                    let query = format!(
                        "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1) as exists",
                        table, column
                    );
                    
                    let value_str = value.as_str().unwrap_or_default();
                    
                    let exists: (bool,) = sqlx::query_as(&query)
                        .bind(value_str)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| ValidationError::new(
                            field_name,
                            "exists",
                            format!("Database error: {}", e),
                        ))?;
                    
                    if !exists.0 {
                        return Err(ValidationError::new(
                            field_name,
                            "exists",
                            format!("{} references a non-existent record", field_name),
                        ));
                    }
                }
                
                // Portable rules are ignored in async context
                _ => {}
            }
            
            Ok(())
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_required_validation() {
        // Empty string fails
        let result = validate_portable("name", &json!(""), &ValidationRule::Required);
        assert!(result.is_err());
        
        // Null fails
        let result = validate_portable("name", &Value::Null, &ValidationRule::Required);
        assert!(result.is_err());
        
        // Non-empty passes
        let result = validate_portable("name", &json!("John"), &ValidationRule::Required);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_length_validation() {
        let result = validate_portable("code", &json!("AB"), &ValidationRule::MinLength(3));
        assert!(result.is_err());
        
        let result = validate_portable("code", &json!("ABCD"), &ValidationRule::MinLength(3));
        assert!(result.is_ok());
        
        let result = validate_portable("code", &json!("ABCDEFGHIJ"), &ValidationRule::MaxLength(5));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_numeric_validation() {
        let result = validate_portable("age", &json!(15), &ValidationRule::Min(18.0));
        assert!(result.is_err());
        
        let result = validate_portable("age", &json!(25), &ValidationRule::Range { min: 18.0, max: 65.0 });
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
    fn test_one_of_validation() {
        let rule = ValidationRule::OneOf { 
            values: vec![json!("active"), json!("pending"), json!("closed")]
        };
        
        let result = validate_portable("status", &json!("active"), &rule);
        assert!(result.is_ok());
        
        let result = validate_portable("status", &json!("invalid"), &rule);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_portable_check() {
        assert!(ValidationRule::Required.is_portable());
        assert!(ValidationRule::Email.is_portable());
        assert!(!ValidationRule::Unique { 
            table: "users".into(), 
            column: "email".into(),
            scope: None,
        }.is_portable());
    }
    
    #[test]
    fn test_validate_all() {
        let rules = vec![
            ValidationRule::Required,
            ValidationRule::MinLength(3),
            ValidationRule::MaxLength(50),
        ];
        
        let errors = validate_all_portable("name", &json!("AB"), &rules);
        assert!(errors.has_errors());
        assert_eq!(errors.errors.len(), 1); // MinLength fails
        
        let errors = validate_all_portable("name", &json!("John Doe"), &rules);
        assert!(!errors.has_errors());
    }
}
