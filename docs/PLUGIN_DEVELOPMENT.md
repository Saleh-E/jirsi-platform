# Jirsi Plugin Development Guide

Build custom workflow nodes and integrations using WASM plugins.

## Quick Start

```bash
# Install wit-bindgen
cargo install wit-bindgen-cli

# Generate bindings from plugin.wit
wit-bindgen rust --out-dir src wit/plugin.wit
```

## Plugin Structure

```rust
use jirsi_plugin::*;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn execute(context: ExecutionContext, input: JsonValue) -> PluginResult {
        // Access host functions
        let entity = host_functions::get_entity("contact", &input["contact_id"])?;
        
        // Log messages
        host_functions::log(LogLevel::Info, "Processing contact");
        
        // Return result
        PluginResult {
            status: ResultStatus::Success,
            data: Some(json!({"processed": true})),
            error_message: None,
        }
    }
    
    fn get_metadata() -> JsonValue {
        json!({
            "name": "My Plugin",
            "version": "1.0.0",
            "description": "Example plugin"
        })
    }
    
    fn validate_config(config: JsonValue) -> Result<bool, String> {
        Ok(true)
    }
}
```

## Host Functions

### Entity Operations
```rust
// Read entity
let contact = get_entity("contact", id)?;

// Query entities
let deals = query_entities("deal", json!({"status": "open"}), 50)?;

// Create entity
let new_task = create_entity("task", json!({"title": "Follow up"}))?;

// Update entity
update_entity("contact", id, json!({"status": "qualified"}))?;

// Delete entity
delete_entity("contact", id)?;
```

### External HTTP
```rust
// Subject to URL allowlist and rate limits
let response = http_fetch(HttpRequest {
    method: "POST".to_string(),
    url: "https://api.stripe.com/v1/charges".to_string(),
    headers: vec![("Authorization", "Bearer sk_xxx")],
    body: Some(body),
})?;
```

### Notifications
```rust
send_notification(
    "email",                    // channel
    "user@example.com",         // recipient
    "welcome_email",            // template_id
    json!({"name": "John"})     // data
)?;
```

## Resource Limits

| Tier | Fuel | Memory | Timeout | HTTP Requests |
|------|------|--------|---------|---------------|
| Untrusted | 100M | 16 MB | 5s | 3 |
| Trusted | 5B | 128 MB | 60s | 50 |
| System | ∞ | 512 MB | 5m | ∞ |

## Building Your Plugin

```bash
# Build for WASM
cargo build --target wasm32-wasi --release

# Optimize size
wasm-opt -O3 target/wasm32-wasi/release/my_plugin.wasm -o my_plugin.opt.wasm
```

## Uploading to Marketplace

1. Create app in Developer Portal
2. Upload compiled WASM module
3. Define input/output schemas
4. Submit for review

## Best Practices

- ✅ Handle errors gracefully
- ✅ Log important operations
- ✅ Validate all inputs
- ✅ Minimize external calls
- ❌ Don't store secrets in code
- ❌ Don't exceed rate limits
