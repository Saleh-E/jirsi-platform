//! LogicBuilder - Visual Rule Builder for Antigravity Logic Engine
//!
//! Provides a dropdown-based UI for building LogicOp rules without writing JSON.
//! Designed for the Node Inspector's Logic Tab.

use leptos::*;
use serde_json::{json, Value};

/// Available LogicOp types for the visual builder
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogicOpType {
    Always,
    Never,
    HasRole,
    Equals,
    NotEquals,
    Empty,
    And,
    Or,
}

impl LogicOpType {
    fn label(&self) -> &'static str {
        match self {
            Self::Always => "Always (Default)",
            Self::Never => "Never",
            Self::HasRole => "Has Role",
            Self::Equals => "Field Equals",
            Self::NotEquals => "Field Not Equals",
            Self::Empty => "Field Is Empty",
            Self::And => "All Conditions (AND)",
            Self::Or => "Any Condition (OR)",
        }
    }
    
    fn from_str(s: &str) -> Self {
        match s {
            "never" => Self::Never,
            "hasRole" => Self::HasRole,
            "equals" => Self::Equals,
            "notEquals" => Self::NotEquals,
            "empty" => Self::Empty,
            "and" => Self::And,
            "or" => Self::Or,
            _ => Self::Always,
        }
    }
}

/// LogicBuilder component - Visual rule editor
#[component]
pub fn LogicBuilder(
    /// Current value as JSON
    #[prop(into)] value: Signal<Value>,
    /// Label for this builder
    #[prop(into)] label: String,
    /// Help text
    #[prop(into)] help: String,
    /// Callback when value changes
    on_change: Callback<Value>,
) -> impl IntoView {
    // Parse current value to determine initial state
    let (op_type, set_op_type) = create_signal(LogicOpType::Always);
    let (field_name, set_field_name) = create_signal(String::new());
    let (field_value, set_field_value) = create_signal(String::new());
    let (role_name, set_role_name) = create_signal(String::new());
    let (show_advanced, set_show_advanced) = create_signal(false);
    
    // Parse initial value
    create_effect(move |_| {
        let val = value.get();
        if let Some(obj) = val.as_object() {
            // Detect op type from value
            if obj.contains_key("Always") || val.as_str() == Some("always") {
                set_op_type.set(LogicOpType::Always);
            } else if obj.contains_key("Never") || val.as_str() == Some("never") {
                set_op_type.set(LogicOpType::Never);
            } else if let Some(role_obj) = obj.get("HasRole") {
                set_op_type.set(LogicOpType::HasRole);
                if let Some(role) = role_obj.get("role").and_then(|r| r.as_str()) {
                    set_role_name.set(role.to_string());
                }
            } else if let Some(eq_obj) = obj.get("Equals") {
                set_op_type.set(LogicOpType::Equals);
                if let Some(field) = eq_obj.get("field").and_then(|f| f.as_str()) {
                    set_field_name.set(field.to_string());
                }
                if let Some(v) = eq_obj.get("value") {
                    set_field_value.set(v.to_string());
                }
            } else if let Some(ne_obj) = obj.get("NotEquals") {
                set_op_type.set(LogicOpType::NotEquals);
                if let Some(field) = ne_obj.get("field").and_then(|f| f.as_str()) {
                    set_field_name.set(field.to_string());
                }
                if let Some(v) = ne_obj.get("value") {
                    set_field_value.set(v.to_string());
                }
            } else if obj.contains_key("Empty") {
                set_op_type.set(LogicOpType::Empty);
                if let Some(empty_obj) = obj.get("Empty") {
                    if let Some(field) = empty_obj.get("field").and_then(|f| f.as_str()) {
                        set_field_name.set(field.to_string());
                    }
                }
            } else if obj.contains_key("And") {
                set_op_type.set(LogicOpType::And);
                set_show_advanced.set(true);
            } else if obj.contains_key("Or") {
                set_op_type.set(LogicOpType::Or);
                set_show_advanced.set(true);
            }
        }
    });
    
    // Build JSON from current state
    let build_json = move || -> Value {
        match op_type.get() {
            LogicOpType::Always => json!("Always"),
            LogicOpType::Never => json!("Never"),
            LogicOpType::HasRole => {
                let role = role_name.get();
                json!({"HasRole": {"role": role}})
            }
            LogicOpType::Equals => {
                let field = field_name.get();
                let val = field_value.get();
                // Try to parse as JSON, fallback to string
                let parsed_val = serde_json::from_str::<Value>(&val)
                    .unwrap_or_else(|_| json!(val));
                json!({"Equals": {"field": field, "value": parsed_val}})
            }
            LogicOpType::NotEquals => {
                let field = field_name.get();
                let val = field_value.get();
                let parsed_val = serde_json::from_str::<Value>(&val)
                    .unwrap_or_else(|_| json!(val));
                json!({"NotEquals": {"field": field, "value": parsed_val}})
            }
            LogicOpType::Empty => {
                let field = field_name.get();
                json!({"Empty": {"field": field}})
            }
            LogicOpType::And => {
                json!({"And": []})
            }
            LogicOpType::Or => {
                json!({"Or": []})
            }
        }
    };
    
    let on_change_clone = on_change.clone();
    
    view! {
        <div class="logic-builder">
            <label class="inspector-label">{label}</label>
            <p class="inspector-help">{help}</p>
            
            // Main condition type selector
            <div class="logic-builder-row">
                <select 
                    class="inspector-select logic-op-select"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_op_type.set(LogicOpType::from_str(&val));
                        // Emit change
                        on_change.call(build_json());
                    }
                >
                    <option value="always" selected=move || op_type.get() == LogicOpType::Always>
                        "✅ Always (Default)"
                    </option>
                    <option value="never" selected=move || op_type.get() == LogicOpType::Never>
                        "🚫 Never"
                    </option>
                    <option value="hasRole" selected=move || op_type.get() == LogicOpType::HasRole>
                        "👤 Has Role"
                    </option>
                    <option value="equals" selected=move || op_type.get() == LogicOpType::Equals>
                        "🔍 Field Equals"
                    </option>
                    <option value="notEquals" selected=move || op_type.get() == LogicOpType::NotEquals>
                        "≠ Field Not Equals"
                    </option>
                    <option value="empty" selected=move || op_type.get() == LogicOpType::Empty>
                        "∅ Field Is Empty"
                    </option>
                    <option value="and" selected=move || op_type.get() == LogicOpType::And>
                        "∧ All Conditions (AND)"
                    </option>
                    <option value="or" selected=move || op_type.get() == LogicOpType::Or>
                        "∨ Any Condition (OR)"
                    </option>
                </select>
            </div>
            
            // Conditional fields based on op type
            {move || match op_type.get() {
                LogicOpType::HasRole => {
                    view! {
                        <div class="logic-builder-params">
                            <label class="param-label">"Role Name"</label>
                            <select 
                                class="inspector-select"
                                on:change=move |ev| {
                                    set_role_name.set(event_target_value(&ev));
                                    on_change_clone.call(build_json());
                                }
                            >
                                <option value="" selected=move || role_name.get().is_empty()>"Select role..."</option>
                                <option value="admin" selected=move || role_name.get() == "admin">"Admin"</option>
                                <option value="manager" selected=move || role_name.get() == "manager">"Manager"</option>
                                <option value="agent" selected=move || role_name.get() == "agent">"Agent"</option>
                                <option value="viewer" selected=move || role_name.get() == "viewer">"Viewer"</option>
                            </select>
                        </div>
                    }.into_view()
                }
                LogicOpType::Equals | LogicOpType::NotEquals => {
                    view! {
                        <div class="logic-builder-params">
                            <div class="param-row">
                                <label class="param-label">"Field"</label>
                                <input 
                                    type="text"
                                    class="inspector-input"
                                    placeholder="e.g., status"
                                    prop:value=field_name
                                    on:input=move |ev| set_field_name.set(event_target_value(&ev))
                                    on:blur=move |_| on_change_clone.call(build_json())
                                />
                            </div>
                            <div class="param-row">
                                <label class="param-label">"Value"</label>
                                <input 
                                    type="text"
                                    class="inspector-input"
                                    placeholder="e.g., active"
                                    prop:value=field_value
                                    on:input=move |ev| set_field_value.set(event_target_value(&ev))
                                    on:blur=move |_| on_change_clone.call(build_json())
                                />
                            </div>
                        </div>
                    }.into_view()
                }
                LogicOpType::Empty => {
                    view! {
                        <div class="logic-builder-params">
                            <label class="param-label">"Field"</label>
                            <input 
                                type="text"
                                class="inspector-input"
                                placeholder="e.g., email"
                                prop:value=field_name
                                on:input=move |ev| set_field_name.set(event_target_value(&ev))
                                on:blur=move |_| on_change_clone.call(build_json())
                            />
                        </div>
                    }.into_view()
                }
                LogicOpType::And | LogicOpType::Or => {
                    view! {
                        <div class="logic-builder-params advanced-notice">
                            <p class="inspector-help">
                                "⚠️ Complex conditions require JSON editing. "
                                "Toggle Advanced Mode below to edit directly."
                            </p>
                            <button 
                                class="btn btn-sm btn-outline"
                                on:click=move |_| set_show_advanced.set(!show_advanced.get())
                            >
                                {move || if show_advanced.get() { "Hide JSON" } else { "Show JSON" }}
                            </button>
                        </div>
                    }.into_view()
                }
                _ => view! { <div></div> }.into_view()
            }}
            
            // Preview of generated JSON
            <div class="logic-builder-preview">
                <details>
                    <summary class="preview-toggle">"📋 View Generated JSON"</summary>
                    <pre class="json-preview">
                        {move || serde_json::to_string_pretty(&build_json()).unwrap_or_default()}
                    </pre>
                </details>
            </div>
        </div>
    }
}
