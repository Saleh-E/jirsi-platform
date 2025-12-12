//! Generic Entity List Page - Metadata-driven
//! 
//! This component renders entity lists dynamically using FieldDefs metadata.
//! Columns and form fields are NOT hardcoded per entity type.

use leptos::*;
use leptos_router::*;
use crate::api::{
    fetch_field_defs, fetch_entity_list, create_entity, FieldDef,
};

/// Main entity list page component
#[component]
pub fn EntityListPage() -> impl IntoView {
    let params = use_params_map();
    let entity_type = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    
    // Signals
    let (fields, set_fields) = create_signal(Vec::<FieldDef>::new());
    let (data, set_data) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(true);
    let (show_form, set_show_form) = create_signal(false);
    let (form_data, set_form_data) = create_signal(serde_json::Map::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    
    // Load metadata and data when entity type changes
    let entity_for_load = entity_type.clone();
    create_effect(move |_| {
        let etype = entity_for_load();
        if etype.is_empty() { return; }
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            // Fetch field definitions
            match fetch_field_defs(&etype).await {
                Ok(f) => set_fields.set(f),
                Err(e) => logging::log!("Failed to fetch fields: {}", e),
            }
            
            // Fetch entity data
            match fetch_entity_list(&etype).await {
                Ok(response) => set_data.set(response.data),
                Err(e) => set_error.set(Some(e)),
            }
            
            set_loading.set(false);
        });
    });
    
    // Get list columns (fields with show_in_list = true)
    let list_columns = move || {
        fields.get()
            .into_iter()
            .filter(|f| f.show_in_list)
            .collect::<Vec<_>>()
    };
    
    // Handle form input change
    let update_form_field = move |field_name: String, value: String| {
        set_form_data.update(|map| {
            map.insert(field_name, serde_json::Value::String(value));
        });
    };
    
    // Handle form submit
    let entity_for_submit = entity_type.clone();
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let etype = entity_for_submit();
        let body = serde_json::Value::Object(form_data.get());
        
        spawn_local(async move {
            match create_entity(&etype, body).await {
                Ok(_) => {
                    set_show_form.set(false);
                    set_form_data.set(serde_json::Map::new());
                    // Refresh list
                    if let Ok(response) = fetch_entity_list(&etype).await {
                        set_data.set(response.data);
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };
    
    // Format entity type for display
    let entity_label = move || {
        let etype = entity_type();
        match etype.as_str() {
            "contact" => "Contacts".to_string(),
            "company" => "Companies".to_string(),
            "deal" => "Deals".to_string(),
            "task" => "Tasks".to_string(),
            "property" => "Properties".to_string(),
            _ => etype.to_uppercase(),
        }
    };
    
    view! {
        <div class="entity-list-page">
            <header class="page-header">
                <h1>{entity_label}</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_form.set(true)>
                    "+ New"
                </button>
            </header>
            
            // Error display
            {move || error.get().map(|e| view! {
                <div class="error-banner">{e}</div>
            })}
            
            // Loading state
            {move || loading.get().then(|| view! {
                <div class="loading">"Loading..."</div>
            })}
            
            // Data table - columns from metadata
            {move || (!loading.get()).then(|| {
                let cols = list_columns();
                view! {
                    <div class="table-container">
                        <table class="data-table">
                            <thead>
                                <tr>
                                    // Dynamic headers from FieldDefs
                                    {cols.iter().map(|f| {
                                        view! { <th>{f.label.clone()}</th> }
                                    }).collect_view()}
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let cols = list_columns();
                                    data.get().into_iter().map(|row| {
                                        view! {
                                            <tr class="clickable-row">
                                                // Dynamic cells from field names
                                                {cols.iter().map(|f| {
                                                    let value = row.get(&f.name)
                                                        .map(|v| format_field_value(v, &f.field_type))
                                                        .unwrap_or_default();
                                                    view! { <td>{value}</td> }
                                                }).collect_view()}
                                            </tr>
                                        }
                                    }).collect_view()
                                }}
                            </tbody>
                        </table>
                        
                        {move || data.get().is_empty().then(|| view! {
                            <div class="empty-state">"No records found. Click + New to create one."</div>
                        })}
                    </div>
                }
            })}
            
            // Add New Form Modal - fields from metadata
            {move || show_form.get().then(|| {
                let cols = fields.get();
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_form.set(false)>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <h2>"Add New " {entity_type()}</h2>
                            <form on:submit=on_submit.clone()>
                                // Dynamic form fields from FieldDefs
                                {cols.iter().filter(|f| !f.is_readonly).map(|field| {
                                    let field_name = field.name.clone();
                                    let field_label = field.label.clone();
                                    let field_type = field.field_type.clone();
                                    let is_required = field.is_required;
                                    let placeholder = field.placeholder.clone().unwrap_or_default();
                                    
                                    view! {
                                        <div class="form-group">
                                            <label>
                                                {field_label}
                                                {is_required.then(|| " *")}
                                            </label>
                                            {render_input_field(field_name, field_type, is_required, placeholder, update_form_field.clone())}
                                        </div>
                                    }
                                }).collect_view()}
                                
                                <div class="form-actions">
                                    <button type="button" class="btn" on:click=move |_| set_show_form.set(false)>
                                        "Cancel"
                                    </button>
                                    <button type="submit" class="btn btn-primary">"Save"</button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

/// Format a field value for display based on field type
fn format_field_value(value: &serde_json::Value, field_type: &str) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => {
            if field_type == "currency" || field_type == "money" {
                format!("${}", n)
            } else {
                n.to_string()
            }
        }
        serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        other => other.to_string(),
    }
}

/// Render an input field based on field type
fn render_input_field(
    field_name: String,
    field_type: String, 
    is_required: bool,
    placeholder: String,
    on_change: impl Fn(String, String) + Clone + 'static,
) -> impl IntoView {
    let name = field_name.clone();
    let on_input = move |ev: web_sys::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        on_change(name.clone(), target.value());
    };
    
    match field_type.as_str() {
        "email" => view! {
            <input 
                type="email" 
                placeholder=placeholder 
                required=is_required 
                on:input=on_input
            />
        }.into_view(),
        "phone" => view! {
            <input 
                type="tel" 
                placeholder=placeholder 
                required=is_required
                on:input=on_input
            />
        }.into_view(),
        "number" | "integer" | "currency" => view! {
            <input 
                type="number" 
                placeholder=placeholder 
                required=is_required
                on:input=on_input
            />
        }.into_view(),
        "textarea" | "longtext" => view! {
            <textarea 
                placeholder=placeholder 
                required=is_required
                on:input=on_input
            />
        }.into_view(),
        "date" => view! {
            <input 
                type="date" 
                required=is_required
                on:input=on_input
            />
        }.into_view(),
        "select" => view! {
            <select required=is_required on:change=on_input>
                <option value="">"-- Select --"</option>
            </select>
        }.into_view(),
        _ => view! {
            <input 
                type="text" 
                placeholder=placeholder 
                required=is_required
                on:input=on_input
            />
        }.into_view(),
    }
}
