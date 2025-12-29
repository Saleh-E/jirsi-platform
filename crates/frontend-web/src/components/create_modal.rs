//! Generic Create Modal - Metadata-driven form in a modal for creating new records

use leptos::*;
use crate::api::{fetch_field_defs, post_json, add_field_option, delete_field_option, FieldDef, API_BASE, TENANT_ID};
use crate::components::field_renderer::LinkInput;
use crate::components::smart_select::{SmartSelect, MultiSelect, SelectOption};

/// Record info returned from CreateModal when a new record is created
#[derive(Clone, Debug)]
pub struct CreatedRecord {
    /// The ID of the newly created record
    pub id: String,
    /// Display name/title of the record
    pub display_name: String,
    /// Entity type (e.g., "contact", "property")
    pub entity_type: String,
}

#[component]
pub fn CreateModal(
    entity_type: String,
    entity_label: String,
    #[prop(into)] on_close: Callback<()>,
    /// Callback with full record info when created (id, display_name, entity_type)
    #[prop(into)] on_created: Callback<CreatedRecord>,
    /// Z-index for modal stacking (default 1000, nested modals get +100)
    #[prop(optional, default = 1000)] z_index: i32,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type.clone());
    let entity_label_stored = store_value(entity_label.clone());
    let z_index_stored = store_value(z_index);
    
    // State
    let (fields, set_fields) = create_signal::<Vec<FieldDef>>(Vec::new());
    let (form_data, set_form_data) = create_signal::<std::collections::HashMap<String, String>>(std::collections::HashMap::new());
    let (loading, set_loading) = create_signal(true);
    let (saving, set_saving) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    // Fetch field definitions
    let entity_for_effect = entity_type.clone();
    create_effect(move |_| {
        let et = entity_for_effect.clone();
        
        spawn_local(async move {
            set_loading.set(true);
            
            match fetch_field_defs(&et).await {
                Ok(field_defs) => {
                    set_fields.set(field_defs);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Handle form submit
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let et = entity_type_stored.get_value();
        let data = form_data.get();
        
        set_saving.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            // Convert form data to JSON
            let mut json_data = serde_json::Map::new();
            for (key, value) in data.iter() {
                if !value.is_empty() {
                    json_data.insert(key.clone(), serde_json::json!(value));
                }
            }
            
            let url = format!("{}/entities/{}?tenant_id={}", API_BASE, et.clone(), TENANT_ID);
            
            match post_json::<_, serde_json::Value>(&url, &serde_json::Value::Object(json_data.clone())).await {
                Ok(response) => {
                    if let Some(id) = response.get("id").and_then(|v| v.as_str()) {
                        // Extract display name from response or form data
                        // Try common display fields: name, title, first_name + last_name
                        let display_name = response.get("name")
                            .or_else(|| response.get("title"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| {
                                // Try first_name + last_name combo
                                let first = response.get("first_name")
                                    .or_else(|| json_data.get("first_name"))
                                    .and_then(|v| v.as_str())?;
                                let last = response.get("last_name")
                                    .or_else(|| json_data.get("last_name"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                Some(format!("{} {}", first, last).trim().to_string())
                            })
                            .or_else(|| {
                                // Fallback to reference field (for properties)
                                json_data.get("reference")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            })
                            .unwrap_or_else(|| format!("New {}", et));
                        
                        on_created.call(CreatedRecord {
                            id: id.to_string(),
                            display_name,
                            entity_type: et.clone(),
                        });
                    }
                    set_saving.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_saving.set(false);
                }
            }
        });
    };
    
    // Handle field change
    let update_field = move |name: String, value: String| {
        set_form_data.update(|d| {
            d.insert(name, value);
        });
    };
    
    view! {
        <div 
            class="ui-modal-overlay" 
            style=move || format!("z-index: {}", z_index_stored.get_value())
            on:click=move |_| on_close.call(())
        >
            <div class="ui-modal" on:click=move |ev| ev.stop_propagation()>
                <div class="ui-modal-header">
                    <h2 class="ui-modal-title">{format!("Create {}", entity_label_stored.get_value())}</h2>
                    <button class="ui-btn ui-btn-ghost" on:click=move |_| on_close.call(())>"×"</button>
                </div>
                
                {move || {
                    if loading.get() {
                        view! { <div class="modal-loading">"Loading form..."</div> }.into_view()
                    } else {
                        view! {
                            <form class="modal-form" on:submit=handle_submit>
                                {move || error.get().map(|e| view! { <div class="form-error">{e}</div> })}
                                
                                <div class="form-fields">
                                    <For
                                        each=move || fields.get().into_iter().filter(|f| !f.is_readonly)
                                        key=|f| f.name.clone()
                                        children=move |field| {
                                            let _field_name = field.name.clone();
                                            let field_name_change = field.name.clone();
                                            let field_label = field.label.clone();
                                            let field_type = field.get_field_type();
                                            let is_required = field.is_required;
                                            let placeholder = field.placeholder.clone().unwrap_or_default();
                                            
                                            view! {
                                                <div class="form-field">
                                                    <label class="field-label">
                                                        {field_label}
                                                        {if is_required { " *" } else { "" }}
                                                    </label>
                                                    {match field_type.as_str() {
                                                        "textarea" | "longtext" => view! {
                                                            <textarea
                                                                class="field-input"
                                                                placeholder=placeholder
                                                                required=is_required
                                                                on:input=move |ev| {
                                                                    update_field(field_name_change.clone(), event_target_value(&ev));
                                                                }
                                                            ></textarea>
                                                        }.into_view(),
                                                        "select" | "status" => {
                                                            // Use field.get_options() which handles both array formats
                                                            let options: Vec<SelectOption> = field.get_options()
                                                                .into_iter()
                                                                .map(|(value, label)| SelectOption::new(value, label))
                                                                .collect();
                                                            let field_name_select = field_name_change.clone();
                                                            let field_name_create = field_name_change.clone();
                                                            let field_id = field.id.clone();
                                                            let entity_for_api = entity_type_stored.get_value();
                                                            view! {
                                                                <SmartSelect
                                                                    options=options
                                                                    value=String::new()
                                                                    on_change=move |val: String| {
                                                                        update_field(field_name_select.clone(), val);
                                                                    }
                                                                    allow_search=true
                                                                    allow_create=true
                                                                    on_create_value=Callback::new({
                                                                        let field_id = field_id.clone();
                                                                        let entity = entity_for_api.clone();
                                                                        move |new_val: String| {
                                                                            update_field(field_name_create.clone(), new_val.clone());
                                                                            
                                                                            // Persist new option to backend
                                                                            if !field_id.is_empty() {
                                                                                let field_id = field_id.clone();
                                                                                let entity = entity.clone();
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
                                                                        let entity = entity_for_api.clone();
                                                                        move |val_to_delete: String| {
                                                                            if !field_id.is_empty() {
                                                                                let field_id = field_id.clone();
                                                                                let entity = entity.clone();
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
                                                        },
                                                        "link" | "lookup" => {
                                                            // Get target entity - check multiple sources:
                                                            // 1. ui_hints.lookup_entity (preferred)
                                                            // 2. field_type.config.target_entity
                                                            // 3. Infer from field name (property_id -> property)
                                                            let target = field.ui_hints.as_ref()
                                                                .and_then(|h| h.get("lookup_entity"))
                                                                .and_then(|v| v.as_str())
                                                                .map(|s| s.to_string())
                                                                .or_else(|| {
                                                                    field.field_type.as_object()
                                                                        .and_then(|obj| obj.get("config"))
                                                                        .and_then(|c| c.get("target_entity"))
                                                                        .and_then(|v| v.as_str())
                                                                        .map(|s| s.to_string())
                                                                })
                                                                .or_else(|| {
                                                                    // Infer from field name: property_id -> property
                                                                    if field.name.ends_with("_id") {
                                                                        Some(field.name.trim_end_matches("_id").to_string())
                                                                    } else {
                                                                        None
                                                                    }
                                                                })
                                                                .unwrap_or_else(|| "contact".to_string());
                                                            let field_name_link = field_name_change.clone();
                                                            let on_link_change = move |id: String| {
                                                                update_field(field_name_link.clone(), id);
                                                            };
                                                            view! {
                                                                <LinkInput
                                                                    target_entity=target
                                                                    on_change=on_link_change
                                                                    placeholder="Select or create...".to_string()
                                                                />
                                                            }.into_view()
                                                        },
                                                        "boolean" => view! {
                                                            <input
                                                                type="checkbox"
                                                                class="field-checkbox"
                                                                on:change=move |ev| {
                                                                    let checked = event_target_checked(&ev);
                                                                    update_field(field_name_change.clone(), checked.to_string());
                                                                }
                                                            />
                                                        }.into_view(),
                                                        "date" => view! {
                                                            <input
                                                                type="date"
                                                                class="field-input"
                                                                required=is_required
                                                                on:input=move |ev| {
                                                                    update_field(field_name_change.clone(), event_target_value(&ev));
                                                                }
                                                            />
                                                        }.into_view(),
                                                        "datetime" => view! {
                                                            <input
                                                                type="datetime-local"
                                                                class="field-input"
                                                                required=is_required
                                                                on:input=move |ev| {
                                                                    update_field(field_name_change.clone(), event_target_value(&ev));
                                                                }
                                                            />
                                                        }.into_view(),
                                                        "number" | "integer" | "currency" => view! {
                                                            <input
                                                                type="number"
                                                                class="field-input"
                                                                placeholder=placeholder
                                                                required=is_required
                                                                on:input=move |ev| {
                                                                    update_field(field_name_change.clone(), event_target_value(&ev));
                                                                }
                                                            />
                                                        }.into_view(),
                                                        "email" => view! {
                                                            <input
                                                                type="email"
                                                                class="field-input"
                                                                placeholder=placeholder
                                                                required=is_required
                                                                on:input=move |ev| {
                                                                    update_field(field_name_change.clone(), event_target_value(&ev));
                                                                }
                                                            />
                                                        }.into_view(),
                                                        "taglist" | "tag_list" | "tags" | "multi_select" => {
                                                            // Multi-select with chips and inline creation - use get_options() for proper parsing
                                                            let select_options: Vec<SelectOption> = field.get_options()
                                                                .into_iter()
                                                                .map(|(value, label)| SelectOption::new(value, label))
                                                                .collect();
                                                            
                                                            let field_name_tags = field_name_change.clone();
                                                            let (local_opts, set_local_opts) = create_signal(select_options);
                                                            
                                                            let handle_tags_change = move |vals: Vec<String>| {
                                                                // Store as JSON array string
                                                                let json_arr = serde_json::to_string(&vals).unwrap_or_default();
                                                                update_field(field_name_tags.clone(), json_arr);
                                                            };
                                                            
                                                            let handle_create_tag = move |new_tag: String| {
                                                                let trimmed = new_tag.trim().to_string();
                                                                if !trimmed.is_empty() {
                                                                    set_local_opts.update(|opts| {
                                                                        if !opts.iter().any(|o| o.value == trimmed) {
                                                                            opts.push(SelectOption::new(trimmed.clone(), trimmed.clone()));
                                                                        }
                                                                    });
                                                                }
                                                            };
                                                            
                                                            view! {
                                                                <MultiSelect
                                                                    options=local_opts.get()
                                                                    on_change=handle_tags_change
                                                                    allow_search=true
                                                                    allow_create=true
                                                                    create_label="+ Add Tag".to_string()
                                                                    on_create_value=handle_create_tag
                                                                    placeholder="Select tags...".to_string()
                                                                />
                                                            }.into_view()
                                                        },
                                                        _ => view! {
                                                            <input
                                                                type="text"
                                                                class="field-input"
                                                                placeholder=placeholder
                                                                required=is_required
                                                                on:input=move |ev| {
                                                                    update_field(field_name_change.clone(), event_target_value(&ev));
                                                                }
                                                            />
                                                        }.into_view(),
                                                    }}
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                                
                                <div class="ui-modal-footer">
                                    <button 
                                        type="button" 
                                        class="ui-btn ui-btn-secondary"
                                        on:click=move |_| on_close.call(())
                                    >
                                        "Cancel"
                                    </button>
                                    <button 
                                        type="submit" 
                                        class="ui-btn ui-btn-primary"
                                        disabled=move || saving.get()
                                    >
                                        {move || if saving.get() { "Saving..." } else { "Create" }}
                                    </button>
                                </div>
                            </form>
                        }.into_view()
                    }
                }}
            </div>
        </div>
    }
}
