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

/// Inline editable field (future enhancement)
#[component]
pub fn EditableFieldValue(
    field: FieldDef,
    value: serde_json::Value,
    #[prop(into)] on_change: Callback<serde_json::Value>,
) -> impl IntoView {
    let (editing, set_editing) = create_signal(false);
    let (current_value, set_current_value) = create_signal(value.clone());
    let field_type = field.field_type.clone();
    
    view! {
        <div 
            class="editable-field"
            on:click=move |_| set_editing.set(true)
        >
            {move || {
                if editing.get() {
                    // Render input based on field type
                    let val = current_value.get().as_str().unwrap_or("").to_string();
                    let on_change_cloned = on_change.clone();
                    view! {
                        <input 
                            type="text"
                            value=val
                            class="field-input"
                            on:blur=move |ev| {
                                set_editing.set(false);
                                let new_val = event_target_value(&ev);
                                let json_val = serde_json::Value::String(new_val);
                                set_current_value.set(json_val.clone());
                                on_change_cloned.call(json_val);
                            }
                        />
                    }.into_view()
                } else {
                    render_field_value(&field_type, &current_value.get(), false).into_view()
                }
            }}
        </div>
    }
}
