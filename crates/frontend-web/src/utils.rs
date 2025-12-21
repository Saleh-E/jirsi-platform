//! Shared utility functions for the frontend

use serde_json::Value;

/// Format a field value for display based on field type
pub fn format_field_display(value: &Value, field_type: &str) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::String(s) if s.is_empty() => "—".to_string(),
        Value::String(s) => {
            match field_type.to_lowercase().as_str() {
                "email" => format!("📧 {}", s),
                "phone" => format!("📞 {}", s),
                "url" => format!("🔗 {}", s),
                _ => s.clone(),
            }
        }
        Value::Number(n) => {
            match field_type.to_lowercase().as_str() {
                "currency" | "money" | "price" => format!("${}", n),
                _ => n.to_string(),
            }
        }
        Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        other => other.to_string(),
    }
}

/// Format a datetime string for display
pub fn format_datetime(iso_string: &str) -> String {
    if iso_string.is_empty() { return "—".to_string(); }
    
    // Simple parsing for ISO 8601 (YYYY-MM-DDTHH:MM:SS...)
    let parts: Vec<&str> = iso_string.split('T').collect();
    if parts.len() < 2 { return iso_string.to_string(); }
    
    let date_part = parts[0];
    let time_part = parts[1];
    
    // Extract HH:MM
    let hhmm = if time_part.len() >= 5 {
        &time_part[0..5]
    } else {
        time_part
    };
    
    format!("{} {}", date_part, hhmm)
}
