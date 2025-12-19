//! Universal Smart Input Component
//!
//! A highly reusable input component that handles all field types with:
//! - Auto-save on blur for text/number fields
//! - Custom dropdown with search for select/link fields
//! - Sticky "Add New" footer with recursive modal support
//! - Mobile-optimized bottom sheet rendering

use leptos::*;
use serde_json::Value;
use crate::api::FieldDef;
use crate::context::mobile::use_mobile;
use crate::components::smart_select::{SmartSelect, MultiSelect, SelectOption};
use crate::components::create_modal::CreateModal;
use crate::api::{fetch_entity_list, fetch_entity_lookup, add_field_option, delete_field_option, API_BASE, TENANT_ID};
use leptos::spawn_local;

/// Input mode for SmartInput
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputMode {
    /// Read-only display mode
    ReadOnly,
    /// Editable mode
    Edit,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::ReadOnly
    }
}

/// Universal Smart Input Component
#[component]
pub fn SmartInput(
    /// Field definition
    field: FieldDef,
    /// Current value
    value: Value,
    /// Callback when value changes
    #[prop(into)] on_change: Callback<Value>,
    /// Input mode (read-only or edit)
    #[prop(optional, default = InputMode::Edit)] mode: InputMode,
    /// Z-index for modals (for stacking)
    #[prop(optional, default = 1000)] z_index: i32,
    /// Entity type name for persisting new options
    #[prop(optional)] entity_type: Option<String>,
) -> impl IntoView {
    let mobile_ctx = use_mobile();
    let _is_mobile = move || mobile_ctx.is_mobile.get();
    
    let field_type = field.get_field_type();
    let field_stored = store_value(field.clone());
    let value_stored = store_value(value.clone());
    let _z_index_stored = store_value(z_index);
    
    // State for editing
    let (is_editing, set_is_editing) = create_signal(mode == InputMode::Edit);
    let (local_value, set_local_value) = create_signal(value.clone());
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    
    // Handle blur for auto-save
    let on_change_blur = on_change.clone();
    let handle_blur = move |_| {
        on_change_blur.call(local_value.get());
        set_is_editing.set(false);
    };
    
    // Handle Enter key
    let on_change_key = on_change.clone();
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            on_change_key.call(local_value.get());
            set_is_editing.set(false);
        } else if ev.key() == "Escape" {
            set_local_value.set(value_stored.get_value());
            set_is_editing.set(false);
        }
    };
    
    // Handle created entity from modal
    let on_change_created = on_change.clone();
    let handle_created = move |record: crate::components::create_modal::CreatedRecord| {
        on_change_created.call(Value::String(record.id));
        set_show_create_modal.set(false);
    };
    
    view! {
        <div class="smart-input" class:editing=move || is_editing.get()>
            {move || {
                let ft = field_stored.get_value().get_field_type();
                let handle_blur = handle_blur.clone();
                let handle_keydown = handle_keydown.clone();
                let on_change = on_change.clone();
                
                match ft.to_lowercase().as_str() {
                    // Text fields
                    "text" | "email" | "phone" | "url" => {
                        if mode == InputMode::ReadOnly && !is_editing.get() {
                            // Read-only display
                            let display = local_value.get().as_str().unwrap_or("").to_string();
                            view! {
                                <div 
                                    class="smart-input-display"
                                    on:click=move |_| set_is_editing.set(true)
                                    title="Click to edit"
                                >
                                    {if display.is_empty() { "—".to_string() } else { display }}
                                </div>
                            }.into_view()
                        } else {
                            // Editable input - use prop:value to prevent focus loss
                            let current = local_value.get_untracked().as_str().unwrap_or("").to_string();
                            let local_value_for_input = local_value;
                            view! {
                                <input
                                    type=ft.clone()
                                    class="smart-input-field"
                                    prop:value=move || local_value_for_input.get().as_str().unwrap_or("").to_string()
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev);
                                        set_local_value.set(Value::String(val));
                                    }
                                    on:blur=handle_blur.clone()
                                    on:keydown=handle_keydown.clone()
                                />
                            }.into_view()
                        }
                    }
                    
                    // Number fields
                    "number" | "integer" | "decimal" | "currency" | "money" => {
                        if mode == InputMode::ReadOnly && !is_editing.get() {
                            let display = match local_value.get() {
                                Value::Number(n) => {
                                    if ft == "currency" || ft == "money" {
                                        format!("${:.2}", n.as_f64().unwrap_or(0.0))
                                    } else {
                                        n.to_string()
                                    }
                                }
                                _ => "—".to_string()
                            };
                            view! {
                                <div 
                                    class="smart-input-display"
                                    on:click=move |_| set_is_editing.set(true)
                                    title="Click to edit"
                                >
                                    {display}
                                </div>
                            }.into_view()
                        } else {
                            let local_value_for_input = local_value;
                            view! {
                                <input
                                    type="number"
                                    class="smart-input-field"
                                    prop:value=move || local_value_for_input.get().as_f64().unwrap_or(0.0).to_string()
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev);
                                        if let Ok(n) = val.parse::<f64>() {
                                            set_local_value.set(serde_json::json!(n));
                                        }
                                    }
                                    on:blur=handle_blur.clone()
                                    on:keydown=handle_keydown.clone()
                                />
                            }.into_view()
                        }
                    }
                    
                    // Select fields  
                    "select" | "status" => {
                        let field_val = field_stored.get_value();
                        let options: Vec<SelectOption> = get_field_options(&field_val);
                        
                        let current = local_value.get().as_str().unwrap_or("").to_string();
                        let on_change = on_change.clone();
                        let on_change_create = on_change.clone();
                        
                        // Get field_id and entity_type for persisting new options
                        let field_id = field_stored.get_value().id.clone();
                        let entity_for_api = entity_type.clone().unwrap_or_default();
                        
                        view! {
                            <SmartSelect
                                options=options
                                value=current
                                on_change=move |val: String| {
                                    set_local_value.set(Value::String(val.clone()));
                                    on_change.call(Value::String(val));
                                }
                                allow_search=true
                                allow_create=true
                                on_create_value=Callback::new({
                                    let field_id = field_id.clone();
                                    let entity_for_api = entity_for_api.clone();
                                    move |new_val: String| {
                                        set_local_value.set(Value::String(new_val.clone()));
                                        on_change_create.call(Value::String(new_val.clone()));
                                        
                                        // Persist new option to backend
                                        if !field_id.is_empty() && !entity_for_api.is_empty() {
                                            let field_id = field_id.clone();
                                            let entity = entity_for_api.clone();
                                            let val = new_val.clone();
                                            spawn_local(async move {
                                                if let Err(e) = add_field_option(&entity, &field_id, &val, Some(&val)).await {
                                                    web_sys::console::error_1(&format!("Failed to persist option: {}", e).into());
                                                }
                                            });
                                        }
                                    }
                                })
                                on_delete_option=Callback::new({
                                    let field_id = field_id.clone();
                                    let entity_for_api = entity_for_api.clone();
                                    move |val_to_delete: String| {
                                        // Persist deletion to backend
                                        if !field_id.is_empty() && !entity_for_api.is_empty() {
                                            let field_id = field_id.clone();
                                            let entity = entity_for_api.clone();
                                            let val = val_to_delete.clone();
                                            spawn_local(async move {
                                                if let Err(e) = delete_field_option(&entity, &field_id, &val).await {
                                                    web_sys::console::error_1(&format!("Failed to delete option: {}", e).into());
                                                }
                                            });
                                        }
                                    }
                                })
                                create_label="+ Add New".to_string()
                                placeholder="Search or type to add...".to_string()
                            />
                        }.into_view()
                    }
                    
                    // TagList / MultiSelect fields - chips with inline tag creation
                    "taglist" | "tag_list" | "tags" | "multi_select" => {
                        let field_val = field_stored.get_value();
                        let options: Vec<SelectOption> = get_field_options(&field_val);
                        
                        // Get current values as array
                        let current_values: Vec<String> = local_value.get()
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        
                        let on_change = on_change.clone();
                        let (local_options, set_local_options) = create_signal(options);
                        
                        // Handle inline tag creation
                        let handle_create_tag = move |new_tag: String| {
                            let trimmed = new_tag.trim().to_string();
                            if !trimmed.is_empty() {
                                // Add to local options
                                set_local_options.update(|opts| {
                                    if !opts.iter().any(|o| o.value == trimmed) {
                                        opts.push(SelectOption::new(trimmed.clone(), trimmed.clone()));
                                    }
                                });
                            }
                        };
                        
                        // Handle multi-select value changes
                        let handle_multi_change = move |vals: Vec<String>| {
                            let json_vals: Vec<Value> = vals.iter()
                                .map(|v| Value::String(v.clone()))
                                .collect();
                            let arr = Value::Array(json_vals);
                            set_local_value.set(arr.clone());
                            on_change.call(arr);
                        };
                        
                        view! {
                            <MultiSelect
                                options=local_options.get()
                                values=current_values
                                on_change=handle_multi_change
                                allow_search=true
                                allow_create=true
                                create_label="+ Add Tag".to_string()
                                on_create_value=handle_create_tag
                            />
                        }.into_view()
                    }
                    // Link fields (references to other entities)
                    "link" => {
                        let field_val = field_stored.get_value();
                        let target_entity = get_link_target(&field_val);
                        
                        view! {
                            <LinkFieldInput
                                target_entity=target_entity
                                value=local_value
                                set_value=set_local_value
                                on_change=on_change.clone()
                                set_show_create_modal=set_show_create_modal
                            />
                        }.into_view()
                    }
                    
                    // MultiLink fields (many-to-many entity relationships)
                    "multilink" | "multi_link" => {
                        let field_val = field_stored.get_value();
                        let target_entity = get_link_target(&field_val);
                        let target_entity_for_fetch = target_entity.clone();
                        let target_entity_for_modal = target_entity.clone();
                        
                        // Get current values as array of IDs
                        let current_ids: Vec<String> = local_value.get()
                            .as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        
                        // State for async-loaded options
                        let (entity_options, set_entity_options) = create_signal::<Vec<SelectOption>>(Vec::new());
                        let (loading, set_loading) = create_signal(true);
                        let (show_create, set_show_create) = create_signal(false);
                        
                        // Fetch records on mount
                        create_effect(move |_| {
                            let entity = target_entity_for_fetch.clone();
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
                                    set_entity_options.set(opts);
                                }
                                set_loading.set(false);
                            });
                        });
                        
                        let on_change = on_change.clone();
                        
                        // Handle multi-select value changes
                        let handle_multi_link_change = move |vals: Vec<String>| {
                            let json_vals: Vec<Value> = vals.iter()
                                .map(|v| Value::String(v.clone()))
                                .collect();
                            let arr = Value::Array(json_vals);
                            set_local_value.set(arr.clone());
                            on_change.call(arr);
                        };
                        
                        // Handle entity created - add to selection and refresh list
                        let target_for_created = target_entity.clone();
                        let handle_multilink_created = move |record: crate::components::create_modal::CreatedRecord| {
                            // Add new entity to options
                            set_entity_options.update(|opts| {
                                opts.push(SelectOption::new(record.id.clone(), record.display_name.clone()));
                            });
                            set_show_create.set(false);
                            
                            // Auto-select the new record (without triggering a full on_change)
                            // Can be enhanced later to auto-add to selection
                        };
                        
                        let create_label_str = format!("+ New {}", target_entity.clone());
                        
                        view! {
                            <div class="multilink-input">
                                {move || {
                                    if loading.get() {
                                        view! { <span class="multilink-loading">"Loading..."</span> }.into_view()
                                    } else {
                                        view! {
                                            <MultiSelect
                                                options=entity_options.get()
                                                values=current_ids.clone()
                                                on_change=handle_multi_link_change.clone()
                                                allow_search=true
                                                allow_create=true
                                                create_label=create_label_str.clone()
                                                on_create=move |_| set_show_create.set(true)
                                            />
                                        }.into_view()
                                    }
                                }}
                                
                                // Create modal for new entities
                                {move || show_create.get().then(|| {
                                    let entity = target_for_created.clone();
                                    let label = entity.clone();
                                    view! {
                                        <CreateModal
                                            entity_type=entity
                                            entity_label=label
                                            on_close=move |_| set_show_create.set(false)
                                            on_created=handle_multilink_created.clone()
                                            z_index=z_index + 100
                                        />
                                    }
                                })}
                            </div>
                        }.into_view()
                    }
                    
                    "boolean" | "checkbox" => {
                        let checked = local_value.get().as_bool().unwrap_or(false);
                        let on_change = on_change.clone();
                        view! {
                            <label class="smart-input-checkbox">
                                <input
                                    type="checkbox"
                                    checked=checked
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        on_change.call(Value::Bool(checked));
                                    }
                                />
                                <span class="checkbox-slider"></span>
                            </label>
                        }.into_view()
                    }
                    
                    // Date fields
                    "date" | "datetime" => {
                        let current = local_value.get().as_str().unwrap_or("").to_string();
                        let on_change = on_change.clone();
                        view! {
                            <input
                                type=if ft == "datetime" { "datetime-local" } else { "date" }
                                class="smart-input-field smart-input-date"
                                value=current
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    on_change.call(Value::String(val));
                                }
                            />
                        }.into_view()
                    }
                    
                    // Textarea
                    "textarea" | "longtext" | "text_area" => {
                        if mode == InputMode::ReadOnly && !is_editing.get() {
                            let display = local_value.get().as_str().unwrap_or("").to_string();
                            view! {
                                <div 
                                    class="smart-input-display smart-input-multiline"
                                    on:click=move |_| set_is_editing.set(true)
                                    title="Click to edit"
                                >
                                    {if display.is_empty() { "—".to_string() } else { display }}
                                </div>
                            }.into_view()
                        } else {
                            let local_value_for_textarea = local_value;
                            view! {
                                <textarea
                                    class="smart-input-field smart-input-textarea"
                                    prop:value=move || local_value_for_textarea.get().as_str().unwrap_or("").to_string()
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev);
                                        set_local_value.set(Value::String(val));
                                    }
                                    on:blur=handle_blur.clone()
                                >
                                </textarea>
                            }.into_view()
                        }
                    }
                    
                    // Default: text input - use prop:value to prevent focus loss
                    _ => {
                        let local_value_for_input = local_value;
                        view! {
                            <input
                                type="text"
                                class="smart-input-field"
                                prop:value=move || local_value_for_input.get().as_str().unwrap_or("").to_string()
                                on:input=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_local_value.set(Value::String(val));
                                }
                                on:blur=handle_blur.clone()
                                on:keydown=handle_keydown.clone()
                            />
                        }.into_view()
                    }
                }
            }}
            
            // Create Modal (for link fields)
            {move || show_create_modal.get().then(|| {
                let field = field_stored.get_value();
                let target = get_link_target(&field);
                let label = capitalize(&target);
                let handle_created = handle_created.clone();
                view! {
                    <CreateModal
                        entity_type=target.clone()
                        entity_label=label
                        on_close=move |_| set_show_create_modal.set(false)
                        on_created=handle_created
                        z_index=z_index + 100
                    />
                }
            })}
        </div>
    }
}

/// Link field input component
#[component]
fn LinkFieldInput(
    target_entity: String,
    value: ReadSignal<Value>,
    set_value: WriteSignal<Value>,
    #[prop(into)] on_change: Callback<Value>,
    set_show_create_modal: WriteSignal<bool>,
) -> impl IntoView {
    let (options, set_options) = create_signal::<Vec<SelectOption>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    
    let target_for_fetch = target_entity.clone();
    let target_for_label = target_entity.clone();
    
    // Fetch options from lookup API (efficient - returns pre-formatted id/label pairs)
    create_effect(move |_| {
        let target = target_for_fetch.clone();
        set_loading.set(true);
        
        spawn_local(async move {
            // Use the new lookup endpoint for better performance
            if let Ok(results) = fetch_entity_lookup(&target, None).await {
                let opts: Vec<SelectOption> = results
                    .into_iter()
                    .map(|r| SelectOption::new(r.id, r.label))
                    .collect();
                set_options.set(opts);
            }
            set_loading.set(false);
        });
    });
    
    let current = value.get().as_str().unwrap_or("").to_string();
    let create_label = format!("+ Add New {}", capitalize(&target_for_label));
    
    view! {
        <div class="smart-input-link">
            {move || {
                if loading.get() {
                    view! { <span class="smart-input-loading">"Loading..."</span> }.into_view()
                } else {
                    let on_change = on_change.clone();
                    let current = current.clone();
                    let create_label = create_label.clone();
                    view! {
                        <SmartSelect
                            options=options.get()
                            value=current
                            on_change=move |val: String| {
                                set_value.set(Value::String(val.clone()));
                                on_change.call(Value::String(val));
                            }
                            allow_search=true
                            allow_create=true
                            create_label=create_label
                            on_create=move |_: ()| set_show_create_modal.set(true)
                        />
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Get options from field definition as SelectOptions
fn get_field_options(field: &FieldDef) -> Vec<SelectOption> {
    field.get_options()
        .into_iter()
        .map(|(value, label)| SelectOption::new(value, label))
        .collect()
}

/// Get link target entity from field definition
fn get_link_target(field: &FieldDef) -> String {
    // First check ui_hints.lookup_entity (preferred)
    if let Some(ui_hints) = &field.ui_hints {
        if let Some(lookup_entity) = ui_hints.get("lookup_entity").and_then(|v| v.as_str()) {
            return lookup_entity.to_string();
        }
    }
    // Then check if field_type is an object with "target" property
    if let Some(obj) = field.field_type.as_object() {
        if let Some(target) = obj.get("target").and_then(|v| v.as_str()) {
            return target.to_string();
        }
    }
    // Infer from field name (property_id -> property, contact_id -> contact)
    if field.name.ends_with("_id") {
        let entity = field.name.trim_end_matches("_id");
        if !entity.is_empty() {
            return entity.to_string();
        }
    }
    // Default to "contact" if not specified
    "contact".to_string()
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
