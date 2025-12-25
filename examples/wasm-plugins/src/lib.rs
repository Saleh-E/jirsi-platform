//! Example WASM Plugin for Jirsi Node Engine
//!
//! This plugin demonstrates how to write custom node logic
//! that can be executed safely in the Jirsi platform.
//!
//! Build with:
//! ```bash
//! cargo build --target wasm32-unknown-unknown --release
//! ```

use extism_pdk::*;
use serde_json::{json, Value};

/// Simple data transformation example
///
/// Takes a JSON object and adds metadata
#[plugin_fn]
pub fn transform(input: Json<Value>) -> FnResult<Json<Value>> {
    let mut data = input.into_inner();
    
    // Add processing metadata
    data["processed"] = json!(true);
    data["processed_at"] = json!(chrono::Utc::now().to_rfc3339());
    data["plugin_version"] = json!("1.0.0");
    
    Ok(Json(data))
}

/// Example: Filter array of records
#[plugin_fn]
pub fn filter_records(input: Json<Value>) -> FnResult<Json<Value>> {
    let data = input.into_inner();
    
    // Expect input like: { "records": [...], "filter": { "field": "status", "value": "active" } }
    let records = data.get("records")
        .and_then(|v| v.as_array())
        .ok_or(Error::msg("Missing 'records' array"))?;
    
    let filter_field = data.get("filter")
        .and_then(|f| f.get("field"))
        .and_then(|f| f.as_str())
        .ok_or(Error::msg("Missing filter.field"))?;
    
    let filter_value = data.get("filter")
        .and_then(|f| f.get("value"))
        .ok_or(Error::msg("Missing filter.value"))?;
    
    // Filter records
    let filtered: Vec<Value> = records.iter()
        .filter(|record| {
            record.get(filter_field)
                .map(|v| v == filter_value)
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    
    Ok(Json(json!({
        "filtered_records": filtered,
        "count": filtered.len(),
        "original_count": records.len(),
    })))
}

/// Example: Calculate sum
#[plugin_fn]
pub fn calculate_sum(input: Json<Value>) -> FnResult<Json<Value>> {
    let data = input.into_inner();
    
    // Expect: { "numbers": [1, 2, 3, 4, 5] }
    let numbers = data.get("numbers")
        .and_then(|v| v.as_array())
        .ok_or(Error::msg("Missing 'numbers' array"))?;
    
    let sum: f64 = numbers.iter()
        .filter_map(|v| v.as_f64())
        .sum();
    
    Ok(Json(json!({
        "sum": sum,
        "count": numbers.len(),
    })))
}

/// Example: String manipulation
#[plugin_fn]
pub fn format_string(input: Json<Value>) -> FnResult<Json<Value>> {
    let data = input.into_inner();
    
    // Expect: { "template": "Hello {name}!", "variables": { "name": "World" } }
    let template = data.get("template")
        .and_then(|v| v.as_str())
        .ok_or(Error::msg("Missing 'template'"))?;
    
    let variables = data.get("variables")
        .and_then(|v| v.as_object())
        .ok_or(Error::msg("Missing 'variables'"))?;
    
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{}}}", key);
        let replacement = value.as_str().unwrap_or("");
        result = result.replace(&placeholder, replacement);
    }
    
    Ok(Json(json!({
        "result": result,
    })))
}

/// Example: Using host functions (logging)
#[plugin_fn]
pub fn log_example(input: Json<Value>) -> FnResult<Json<Value>> {
    let data = input.into_inner();
    
    // Log to host (this will appear in Jirsi server logs)
    log::info!("WASM Plugin received: {:?}", data);
    
    Ok(Json(json!({
        "logged": true,
        "message": "Check server logs",
    })))
}
