//! Field Value Renderer - Renders field values based on FieldDef type
//! 
//! The Golden Rule: A field is defined once and rendered correctly everywhere.
//! This component handles all field type rendering automatically.

use leptos::*;
use crate::api::FieldDef;

/// Renders a field value based on its FieldDef type
/// This is the core component for metadata-driven rendering
#[component]
pub fn FieldValueRenderer(
    field: FieldDef,
    value: serde_json::Value,
    #[prop(optional)] editable: bool,
) -> impl IntoView {
    let field_type = field.field_type.clone();
    
    view! {
        {move || {
            render_field_value(&field_type, &value, editable)
        }}
    }
}

/// Render field value based on type
fn render_field_value(field_type: &str, value: &serde_json::Value, _editable: bool) -> impl IntoView {
    match field_type {
        // Image - render as img tag
        "image" => {
            let url = value.as_str().unwrap_or("").to_string();
            view! {
                <img 
                    src=url.clone() 
                    alt="Image" 
                    class="field-image"
                    style="max-width: 80px; max-height: 60px; border-radius: 4px;"
                />
            }.into_view()
        }
        
        // Money - format with currency
        "money" => {
            let amount = value.as_f64().unwrap_or(0.0);
            let formatted = format!("${:.2}", amount);
            view! {
                <span class="field-money">{formatted}</span>
            }.into_view()
        }
        
        // Select - render as badge
        "select" => {
            let val = value.as_str().unwrap_or("").to_string();
            let badge_class = get_status_color(&val);
            view! {
                <span class=format!("badge {}", badge_class)>{val}</span>
            }.into_view()
        }
        
        // MultiSelect - render as multiple badges
        "multi_select" => {
            let items: Vec<String> = value.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            view! {
                <div class="badge-list">
                    {items.into_iter().map(|item| view! {
                        <span class="badge badge-secondary">{item}</span>
                    }).collect_view()}
                </div>
            }.into_view()
        }
        
        // Link - render as clickable link
        "link" => {
            // For now, just show the ID - in practice this would resolve to a name
            let id = value.as_str().unwrap_or("").to_string();
            let short_id = if id.len() > 8 { &id[..8] } else { &id };
            view! {
                <a href=format!("#/record/{}", id) class="field-link">
                    {short_id.to_string()}"..."
                </a>
            }.into_view()
        }
        
        // TagList - render as chips
        "tag_list" => {
            let tags: Vec<String> = value.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            view! {
                <div class="tag-list">
                    {tags.into_iter().map(|tag| view! {
                        <span class="tag">{tag}</span>
                    }).collect_view()}
                </div>
            }.into_view()
        }
        
        // Boolean - render as checkbox icon
        "boolean" => {
            let checked = value.as_bool().unwrap_or(false);
            let icon = if checked { "✓" } else { "✗" };
            let class = if checked { "bool-true" } else { "bool-false" };
            view! {
                <span class=format!("field-bool {}", class)>{icon}</span>
            }.into_view()
        }
        
        // Date - format nicely
        "date" => {
            let date_str = value.as_str().unwrap_or("").to_string();
            view! {
                <span class="field-date">{date_str}</span>
            }.into_view()
        }
        
        // DateTime - format with time
        "date_time" => {
            let datetime_str = value.as_str().unwrap_or("").to_string();
            // Truncate to readable format
            let display = if datetime_str.len() > 16 {
                datetime_str[..16].replace("T", " ")
            } else {
                datetime_str.clone()
            };
            view! {
                <span class="field-datetime">{display}</span>
            }.into_view()
        }
        
        // Email - render as mailto link
        "email" => {
            let email = value.as_str().unwrap_or("").to_string();
            view! {
                <a href=format!("mailto:{}", email) class="field-email">{email.clone()}</a>
            }.into_view()
        }
        
        // Phone - render as tel link
        "phone" => {
            let phone = value.as_str().unwrap_or("").to_string();
            view! {
                <a href=format!("tel:{}", phone) class="field-phone">{phone.clone()}</a>
            }.into_view()
        }
        
        // URL - render as external link
        "url" => {
            let url = value.as_str().unwrap_or("").to_string();
            view! {
                <a href=url.clone() target="_blank" class="field-url">
                    {if url.len() > 30 { format!("{}...", &url[..30]) } else { url.clone() }}
                </a>
            }.into_view()
        }
        
        // Score - render as progress bar
        "score" => {
            let score = value.as_i64().unwrap_or(0) as i32;
            let percentage = (score.min(100).max(0)) as f64;
            view! {
                <div class="field-score">
                    <div class="score-bar" style=format!("width: {}%", percentage)></div>
                    <span class="score-value">{score}</span>
                </div>
            }.into_view()
        }
        
        // Number - format with decimals
        "number" | "integer" | "decimal" => {
            let num = value.as_f64().unwrap_or(0.0);
            view! {
                <span class="field-number">{format!("{}", num)}</span>
            }.into_view()
        }
        
        // Default: Text, TextArea, RichText, etc.
        _ => {
            let text = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "".to_string(),
                _ => value.to_string(),
            };
            // Truncate long text in list views
            let display = if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text
            };
            view! {
                <span class="field-text">{display}</span>
            }.into_view()
        }
    }
}

/// Get CSS class for status badges based on common status values
fn get_status_color(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        // Green statuses
        "active" | "completed" | "won" | "sold" | "approved" | "accepted" => "badge-success",
        // Red statuses
        "inactive" | "cancelled" | "lost" | "rejected" | "expired" => "badge-danger",
        // Yellow statuses
        "pending" | "draft" | "new" | "negotiation" | "in_progress" => "badge-warning",
        // Blue statuses
        "available" | "open" | "scheduled" => "badge-info",
        // Default
        _ => "badge-secondary",
    }
}

/// Inline editable field - click to edit, supports all field types
#[component]
pub fn EditableFieldValue(
    field: FieldDef,
    value: serde_json::Value,
    #[prop(into)] on_change: Callback<serde_json::Value>,
) -> impl IntoView {
    let (editing, set_editing) = create_signal(false);
    let (current_value, set_current_value) = create_signal(value.clone());
    let field_type = field.field_type.clone();
    let field_type_for_edit = field_type.clone();
    
    view! {
        <div 
            class="editable-field"
            on:dblclick=move |_| set_editing.set(true)
        >
            {move || {
                let ft = field_type_for_edit.clone();
                if editing.get() {
                    render_edit_input(&ft, current_value.get(), set_editing, set_current_value, on_change.clone()).into_view()
                } else {
                    view! {
                        <div class="editable-field-view" title="Double-click to edit">
                            {render_field_value(&field_type, &current_value.get(), false)}
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Render the appropriate input control for editing based on field type
fn render_edit_input(
    field_type: &str,
    current_value: serde_json::Value,
    set_editing: WriteSignal<bool>,
    set_current_value: WriteSignal<serde_json::Value>,
    on_change: Callback<serde_json::Value>,
) -> impl IntoView {
    match field_type {
        // Boolean - render as checkbox
        "boolean" => {
            let checked = current_value.as_bool().unwrap_or(false);
            view! {
                <input 
                    type="checkbox"
                    checked=checked
                    class="field-input-checkbox"
                    on:change=move |ev| {
                        let new_val = event_target_checked(&ev);
                        let json_val = serde_json::Value::Bool(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                        set_editing.set(false);
                    }
                />
            }.into_view()
        }
        
        // Number/Money - render as number input
        "number" | "money" | "integer" | "decimal" | "score" => {
            let num = current_value.as_f64().unwrap_or(0.0);
            view! {
                <input 
                    type="number"
                    value=num.to_string()
                    step="any"
                    class="field-input-number"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        if let Ok(n) = new_val.parse::<f64>() {
                            let json_val = serde_json::json!(n);
                            set_current_value.set(json_val.clone());
                            on_change.call(json_val);
                        }
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" || ev.key() == "Escape" {
                            set_editing.set(false);
                        }
                    }
                />
            }.into_view()
        }
        
        // Date - render as date input
        "date" => {
            let date_str = current_value.as_str().unwrap_or("").to_string();
            view! {
                <input 
                    type="date"
                    value=date_str
                    class="field-input-date"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                />
            }.into_view()
        }
        
        // DateTime - render as datetime-local input
        "date_time" => {
            let datetime_str = current_value.as_str().unwrap_or("").to_string();
            view! {
                <input 
                    type="datetime-local"
                    value=datetime_str
                    class="field-input-datetime"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                />
            }.into_view()
        }
        
        // TextArea - render as textarea for multiline
        "text_area" | "rich_text" => {
            let text = current_value.as_str().unwrap_or("").to_string();
            view! {
                <textarea 
                    class="field-input-textarea"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                >{text}</textarea>
            }.into_view()
        }
        
        // URL/Email/Phone - specialized text inputs
        "url" => {
            let val = current_value.as_str().unwrap_or("").to_string();
            view! {
                <input 
                    type="url"
                    value=val
                    class="field-input-url"
                    placeholder="https://..."
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                />
            }.into_view()
        }
        
        "email" => {
            let val = current_value.as_str().unwrap_or("").to_string();
            view! {
                <input 
                    type="email"
                    value=val
                    class="field-input-email"
                    placeholder="email@example.com"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                />
            }.into_view()
        }
        
        "phone" => {
            let val = current_value.as_str().unwrap_or("").to_string();
            view! {
                <input 
                    type="tel"
                    value=val
                    class="field-input-phone"
                    placeholder="+1-555-000-0000"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                />
            }.into_view()
        }
        
        // Default: Text input for all other types
        _ => {
            let val = current_value.as_str().unwrap_or("").to_string();
            view! {
                <input 
                    type="text"
                    value=val
                    class="field-input-text"
                    on:blur=move |ev| {
                        set_editing.set(false);
                        let new_val = event_target_value(&ev);
                        let json_val = serde_json::Value::String(new_val);
                        set_current_value.set(json_val.clone());
                        on_change.call(json_val);
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" || ev.key() == "Escape" {
                            set_editing.set(false);
                        }
                    }
                />
            }.into_view()
        }
    }
}

