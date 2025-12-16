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
use crate::components::smart_select::{SmartSelect, SelectOption};
use crate::components::create_modal::CreateModal;
use crate::api::{fetch_entity_list, API_BASE, TENANT_ID};

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
    let handle_created = move |id: String| {
        on_change_created.call(Value::String(id));
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
                            // Editable input
                            let current = local_value.get().as_str().unwrap_or("").to_string();
                            view! {
                                <input
                                    type=ft.clone()
                                    class="smart-input-field"
                                    value=current
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
                            let current = local_value.get().as_f64().unwrap_or(0.0).to_string();
                            view! {
                                <input
                                    type="number"
                                    class="smart-input-field"
                                    value=current
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
                        let options: Vec<SelectOption> = get_field_options(&field_val)
                            .into_iter()
                            .map(|opt| SelectOption::new(opt.clone(), opt))
                            .collect();
                        
                        let current = local_value.get().as_str().unwrap_or("").to_string();
                        let on_change = on_change.clone();
                        
                        view! {
                            <SmartSelect
                                options=options
                                value=current
                                on_change=move |val: String| {
                                    set_local_value.set(Value::String(val.clone()));
                                    on_change.call(Value::String(val));
                                }
                                allow_search=true
                                allow_create=false
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
                    
                    // Boolean
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
                            let current = local_value.get().as_str().unwrap_or("").to_string();
                            view! {
                                <textarea
                                    class="smart-input-field smart-input-textarea"
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev);
                                        set_local_value.set(Value::String(val));
                                    }
                                    on:blur=handle_blur.clone()
                                >
                                    {current}
                                </textarea>
                            }.into_view()
                        }
                    }
                    
                    // Default: text input
                    _ => {
                        let current = local_value.get().as_str().unwrap_or("").to_string();
                        view! {
                            <input
                                type="text"
                                class="smart-input-field"
                                value=current
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
    
    // Fetch options from API
    create_effect(move |_| {
        let target = target_for_fetch.clone();
        set_loading.set(true);
        
        spawn_local(async move {
            let url = format!("{}/entities/{}/records?tenant_id={}", API_BASE, target, TENANT_ID);
            if let Ok(response) = fetch_entity_list(&url).await {
                let opts: Vec<SelectOption> = response.data.iter()
                    .filter_map(|r| {
                        let id = r.get("id")?.as_str()?.to_string();
                        let name = r.get("name")
                            .or_else(|| r.get("title"))
                            .or_else(|| r.get("full_name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&id)
                            .to_string();
                        Some(SelectOption::new(id, name))
                    })
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

/// Get options from field definition
fn get_field_options(field: &FieldDef) -> Vec<String> {
    if let Some(ref opts) = field.options {
        if let Some(arr) = opts.as_array() {
            return arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    vec![]
}

/// Get link target entity from field definition
fn get_link_target(field: &FieldDef) -> String {
    // Check if field_type is an object with "target" property
    if let Some(obj) = field.field_type.as_object() {
        if let Some(target) = obj.get("target").and_then(|v| v.as_str()) {
            return target.to_string();
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
