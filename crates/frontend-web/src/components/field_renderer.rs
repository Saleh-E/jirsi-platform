//! Field Value Renderer - Renders field values based on FieldDef type
//! 
//! The Golden Rule: A field is defined once and rendered correctly everywhere.
//! This component handles all field type rendering automatically.

use leptos::*;
use crate::api::{FieldDef, fetch_entity_lookup};
use crate::components::smart_select::{SmartSelect, SelectOption};

// ============================================================================
// FIELD MODE - Defines rendering modes for fields
// ============================================================================

/// Mode for field rendering in different contexts
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum FieldMode {
    /// Read-only display mode (default)
    #[default]
    Read,
    /// Full edit mode (in forms)
    Edit,
    /// Inline edit mode (in tables, activated on dblclick)
    InlineEdit,
}

// ============================================================================
// ASYNC ENTITY SELECT - Uses lookup API for Link field dropdowns
// ============================================================================

/// Async entity select - fetches options from lookup endpoint
/// This is the CRITICAL component for Link field dropdowns
#[component]
pub fn AsyncEntitySelect(
    /// Target entity type (e.g., "property", "contact")
    target_entity: String,
    /// Currently selected value (UUID)
    #[prop(optional)] value: Option<String>,
    /// Callback when selection changes
    #[prop(into)] on_change: Callback<String>,
    /// Placeholder text
    #[prop(optional)] placeholder: Option<String>,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let (options, set_options) = create_signal::<Vec<SelectOption>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (selected_value, set_selected_value) = create_signal(value.clone());
    
    let target_entity_stored = store_value(target_entity.clone());
    
    // Fetch all options on mount (SmartSelect handles client-side filtering)
    create_effect(move |_| {
        let entity = target_entity_stored.get_value();
        spawn_local(async move {
            set_loading.set(true);
            match fetch_entity_lookup(&entity, None).await {
                Ok(results) => {
                    let opts: Vec<SelectOption> = results
                        .into_iter()
                        .map(|r| SelectOption::new(r.id, r.label))
                        .collect();
                    set_options.set(opts);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Lookup error: {}", e).into());
                    set_options.set(vec![]);
                }
            }
            set_loading.set(false);
        });
    });
    
    // Handle selection change
    let handle_change = move |val: String| {
        set_selected_value.set(Some(val.clone()));
        on_change.call(val);
    };
    
    view! {
        <div class="async-entity-select">
            {move || {
                if loading.get() && options.get().is_empty() {
                    view! { <span class="loading-indicator">"Loading..."</span> }.into_view()
                } else {
                    view! {
                        <SmartSelect
                            options=options.get()
                            value=selected_value.get().unwrap_or_default()
                            on_change=handle_change
                            allow_search=true
                            placeholder=placeholder.clone().unwrap_or_else(|| "Select...".to_string())
                            disabled=disabled
                        />
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Renders a field value based on its FieldDef type
/// This is the core component for metadata-driven rendering
#[component]
pub fn FieldValueRenderer(
    field: FieldDef,
    value: serde_json::Value,
    #[prop(optional)] editable: bool,
) -> impl IntoView {
    let field_type = field.get_field_type();
    
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
    let field_type = field.get_field_type();
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

// ============================================================
// Smart Input Components for Inline Creation
// ============================================================

use crate::components::create_modal::CreateModal;
use crate::api::{fetch_entity_list, add_field_option, API_BASE, TENANT_ID};
use wasm_bindgen::JsCast;

/// Created record info returned from CreateModal
#[derive(Clone, Debug)]
pub struct CreatedRecord {
    pub id: String,
    pub display_name: String,
}

/// LinkInput - Smart select for Link fields with inline creation
/// Fetches records from target entity and allows creating new ones
#[component]
pub fn LinkInput(
    /// Target entity type (e.g., "property", "contact")
    target_entity: String,
    /// Target entity label for display
    #[prop(optional)] target_label: Option<String>,
    /// Currently selected value (UUID)
    #[prop(optional)] value: Option<String>,
    /// Callback when selection changes
    #[prop(into)] on_change: Callback<String>,
    /// Placeholder text
    #[prop(optional)] placeholder: Option<String>,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    // State
    let (options, set_options) = create_signal::<Vec<SelectOption>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (show_modal, set_show_modal) = create_signal(false);
    let (selected_value, set_selected_value) = create_signal(value.clone());
    
    let target_entity_stored = store_value(target_entity.clone());
    let target_label_display = target_label.clone().unwrap_or_else(|| {
        // Capitalize entity name for label
        let mut chars = target_entity.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    });
    let target_label_stored = store_value(target_label_display.clone());
    
    // Fetch records on mount
    let fetch_target = target_entity.clone();
    create_effect(move |_| {
        let entity = fetch_target.clone();
        spawn_local(async move {
            set_loading.set(true);
            match fetch_entity_list(&entity).await {
                Ok(records) => {
                    let opts: Vec<SelectOption> = records.data.iter().filter_map(|r: &serde_json::Value| {
                        let id = r.get("id")?.as_str()?.to_string();
                        // Try common display fields
                        let label = r.get("name")
                            .or_else(|| r.get("title"))
                            .or_else(|| r.get("first_name"))
                            .and_then(|v: &serde_json::Value| v.as_str())
                            .map(|s: &str| {
                                // Combine first_name + last_name if available
                                if let Some(last) = r.get("last_name").and_then(|v: &serde_json::Value| v.as_str()) {
                                    format!("{} {}", s, last)
                                } else {
                                    s.to_string()
                                }
                            })
                            .unwrap_or_else(|| id[..8.min(id.len())].to_string());
                        Some(SelectOption::new(id, label))
                    }).collect();
                    set_options.set(opts);
                    set_loading.set(false);
                }
                Err(_) => {
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Handle selection change
    let handle_change = move |val: String| {
        set_selected_value.set(Some(val.clone()));
        on_change.call(val);
    };
    
    // Handle new record created
    let handle_created = move |record: crate::components::create_modal::CreatedRecord| {
        // Immediately select the new record
        set_selected_value.set(Some(record.id.clone()));
        on_change.call(record.id.clone());
        set_show_modal.set(false);
        
        // Refresh the options list
        let entity = target_entity_stored.get_value();
        spawn_local(async move {
            if let Ok(records) = fetch_entity_list(&entity).await {
                let opts: Vec<SelectOption> = records.data.iter().filter_map(|r: &serde_json::Value| {
                    let id = r.get("id")?.as_str()?.to_string();
                    let label = r.get("name")
                        .or_else(|| r.get("title"))
                        .or_else(|| r.get("first_name"))
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(|s: &str| s.to_string())
                        .unwrap_or_else(|| id[..8.min(id.len())].to_string());
                    Some(SelectOption::new(id, label))
                }).collect();
                set_options.set(opts);
            }
        });
    };
    
    view! {
        <div class="link-input">
            {move || {
                if loading.get() {
                    view! { <span class="link-input-loading">"Loading..."</span> }.into_view()
                } else {
                    let create_label_text = format!("+ Add New {}", target_label_stored.get_value());
                    view! {
                        <SmartSelect
                            options=options.get()
                            value=selected_value.get().unwrap_or_default()
                            on_change=handle_change
                            allow_search=true
                            allow_create=true
                            create_label=create_label_text
                            on_create=move |_| set_show_modal.set(true)
                            placeholder=placeholder.clone().unwrap_or_default()
                            disabled=disabled
                        />
                    }.into_view()
                }
            }}
            
            // Create Modal (stacked)
            {move || show_modal.get().then(|| {
                let entity = target_entity_stored.get_value();
                let label = target_label_stored.get_value();
                view! {
                    <CreateModal
                        entity_type=entity
                        entity_label=label
                        on_close=move |_| set_show_modal.set(false)
                        on_created=handle_created
                    />
                }
            })}
        </div>
    }
}

/// DynamicSelect - Smart select for Select fields with option to add new choices
#[component]
pub fn DynamicSelect(
    /// Available options
    options: Vec<String>,
    /// Currently selected value
    #[prop(optional)] value: Option<String>,
    /// Callback when selection changes
    #[prop(into)] on_change: Callback<String>,
    /// Field ID for updating options via API
    #[prop(optional)] field_id: Option<String>,
    /// Entity type name for API call
    #[prop(optional)] entity_type: Option<String>,
    /// Allow adding new options
    #[prop(optional, default = true)] allow_create: bool,
    /// Placeholder text
    #[prop(optional)] placeholder: Option<String>,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let (current_options, set_current_options) = create_signal(options.clone());
    let (selected_value, set_selected_value) = create_signal(value.clone());
    let field_id_stored = store_value(field_id);
    let entity_type_stored = store_value(entity_type);
    
    // Convert string options to SelectOption
    let select_options = move || {
        current_options.get().into_iter().map(|s| SelectOption::new(s.clone(), s)).collect::<Vec<_>>()
    };
    
    // Handle selection
    let handle_change = move |val: String| {
        set_selected_value.set(Some(val.clone()));
        on_change.call(val);
    };
    
    // Handle adding a new option from typed text
    let handle_create_value = move |new_value: String| {
        let trimmed = new_value.trim().to_string();
        if !trimmed.is_empty() {
            // Add to local options immediately
            set_current_options.update(|opts| {
                if !opts.contains(&trimmed) {
                    opts.push(trimmed.clone());
                }
            });
            // Select the new option
            set_selected_value.set(Some(trimmed.clone()));
            on_change.call(trimmed.clone());
            
            // Persist to API if field_id and entity_type are provided
            if let Some(fid) = field_id_stored.get_value() {
                if let Some(entity) = entity_type_stored.get_value() {
                    let new_opt = trimmed.clone();
                    let field_id = fid.clone();
                    let entity_name = entity.clone();
                    spawn_local(async move {
                        if let Err(e) = add_field_option(&entity_name, &field_id, &new_opt, Some(&new_opt)).await {
                            web_sys::console::error_1(&format!("Failed to persist option: {}", e).into());
                        }
                    });
                }
            }
        }
    };
    
    view! {
        <div class="dynamic-select">
            <SmartSelect
                options=select_options()
                value=selected_value.get().unwrap_or_default()
                on_change=handle_change
                allow_search=true
                allow_create=allow_create
                create_label="+ Add New Option".to_string()
                on_create_value=handle_create_value
                placeholder=placeholder.clone().unwrap_or_default()
                disabled=disabled
            />
        </div>
    }
}

/// AsyncLinkInput - Link input with debounced async option loading
/// Fetches options as user types with debounce to reduce API calls
#[component]
pub fn AsyncLinkInput(
    /// Target entity type to fetch
    target_entity: String,
    /// Label for the target entity
    #[prop(optional)] target_label: Option<String>,
    /// Currently selected value (ID)
    #[prop(optional)] value: Option<String>,
    /// Callback when selection changes
    #[prop(into)] on_change: Callback<String>,
    /// Debounce delay in milliseconds
    #[prop(optional, default = 300)] debounce_ms: u32,
    /// Placeholder text
    #[prop(optional)] placeholder: Option<String>,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let target_entity_stored = store_value(target_entity.clone());
    let target_label_stored = store_value(target_label.clone().unwrap_or_else(|| target_entity.clone()));
    
    let (options, set_options) = create_signal::<Vec<SelectOption>>(Vec::new());
    let (selected_value, set_selected_value) = create_signal(value.clone());
    let (loading, set_loading) = create_signal(false);
    let (show_modal, set_show_modal) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (debounce_handle, set_debounce_handle) = create_signal::<Option<i32>>(None);
    
    // Initial fetch
    let entity_for_fetch = target_entity.clone();
    create_effect(move |_| {
        let entity = entity_for_fetch.clone();
        spawn_local(async move {
            set_loading.set(true);
            if let Ok(records) = fetch_entity_list(&entity).await {
                let opts: Vec<SelectOption> = records.data.iter().filter_map(|r: &serde_json::Value| {
                    let id = r.get("id")?.as_str()?.to_string();
                    let label = r.get("name")
                        .or_else(|| r.get("title"))
                        .or_else(|| r.get("first_name"))
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(|s: &str| {
                            if let Some(last) = r.get("last_name").and_then(|v: &serde_json::Value| v.as_str()) {
                                format!("{} {}", s, last)
                            } else {
                                s.to_string()
                            }
                        })
                        .unwrap_or_else(|| id[..8.min(id.len())].to_string());
                    Some(SelectOption::new(id, label))
                }).collect();
                set_options.set(opts);
                set_loading.set(false);
            }
        });
    });
    
    // Debounced search function
    let debounce_search = move |query: String| {
        // Cancel previous timeout
        if let Some(handle) = debounce_handle.get() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(handle);
            }
        }
        
        set_search_query.set(query.clone());
        
        // Set new timeout
        if let Some(window) = web_sys::window() {
            let entity = target_entity_stored.get_value();
            let callback = wasm_bindgen::closure::Closure::once_into_js(move || {
                spawn_local(async move {
                    set_loading.set(true);
                    // Add search query parameter if backend supports it
                    let url = if query.is_empty() {
                        format!("{}/entities/{}?tenant_id={}", API_BASE, entity, TENANT_ID)
                    } else {
                        format!("{}/entities/{}?tenant_id={}&search={}", API_BASE, entity, TENANT_ID, query)
                    };
                    
                    // Fetch with search
                    match crate::api::fetch_json::<crate::api::GenericListResponse>(&url).await {
                        Ok(records) => {
                            let opts: Vec<SelectOption> = records.data.iter().filter_map(|r: &serde_json::Value| {
                                let id = r.get("id")?.as_str()?.to_string();
                                let label = r.get("name")
                                    .or_else(|| r.get("title"))
                                    .or_else(|| r.get("first_name"))
                                    .and_then(|v: &serde_json::Value| v.as_str())
                                    .map(|s: &str| s.to_string())
                                    .unwrap_or_else(|| id[..8.min(id.len())].to_string());
                                Some(SelectOption::new(id, label))
                            }).collect();
                            set_options.set(opts);
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Search fetch error: {}", e).into());
                        }
                    }
                    set_loading.set(false);
                });
            });
            
            if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                debounce_ms as i32,
            ) {
                set_debounce_handle.set(Some(handle));
            }
        }
    };
    
    // Handle selection
    let handle_change = move |val: String| {
        set_selected_value.set(Some(val.clone()));
        on_change.call(val);
    };
    
    // Handle record creation
    let handle_created = move |record: crate::components::create_modal::CreatedRecord| {
        set_selected_value.set(Some(record.id.clone()));
        on_change.call(record.id.clone());
        set_show_modal.set(false);
        
        // Refresh options
        let entity = target_entity_stored.get_value();
        spawn_local(async move {
            if let Ok(records) = fetch_entity_list(&entity).await {
                let opts: Vec<SelectOption> = records.data.iter().filter_map(|r: &serde_json::Value| {
                    let id = r.get("id")?.as_str()?.to_string();
                    let label = r.get("name")
                        .or_else(|| r.get("title"))
                        .or_else(|| r.get("first_name"))
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(|s: &str| s.to_string())
                        .unwrap_or_else(|| id[..8.min(id.len())].to_string());
                    Some(SelectOption::new(id, label))
                }).collect();
                set_options.set(opts);
            }
        });
    };
    
    view! {
        <div class="async-link-input">
            {move || {
                if loading.get() && options.get().is_empty() {
                    view! { <span class="link-input-loading">"Loading..."</span> }.into_view()
                } else {
                    let create_label_text = format!("+ Add New {}", target_label_stored.get_value());
                    view! {
                        <SmartSelect
                            options=options.get()
                            value=selected_value.get().unwrap_or_default()
                            on_change=handle_change
                            allow_search=true
                            allow_create=true
                            create_label=create_label_text
                            on_create=move |_| set_show_modal.set(true)
                            placeholder=placeholder.clone().unwrap_or_default()
                            disabled=disabled
                        />
                    }.into_view()
                }
            }}
            
            // Create Modal
            {move || show_modal.get().then(|| {
                let entity = target_entity_stored.get_value();
                let label = target_label_stored.get_value();
                view! {
                    <CreateModal
                        entity_type=entity
                        entity_label=label
                        on_close=move |_| set_show_modal.set(false)
                        on_created=handle_created
                    />
                }
            })}
        </div>
    }
}
