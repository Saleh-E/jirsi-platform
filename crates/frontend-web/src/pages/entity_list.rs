//! Generic Entity List Page - Metadata-driven
//! 
//! This component renders entity lists dynamically using FieldDefs metadata.
//! Columns and form fields are NOT hardcoded per entity type.
//! Supports multiple view types via ViewSwitcher.

use leptos::*;
use leptos_router::*;
use crate::api::{
    fetch_field_defs, fetch_entity_list, create_entity, FieldDef,
};
use crate::components::view_switcher::{ViewSwitcher, ViewDefResponse};
use crate::components::kanban::{KanbanView, KanbanConfig};
use crate::components::calendar::{CalendarView, CalendarConfig};
use crate::components::map::{MapView, MapConfig};
use crate::context::mobile::use_mobile;

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
    
    // View switching state
    let (current_view_type, set_current_view_type) = create_signal("table".to_string());
    let (current_view_settings, set_current_view_settings) = create_signal(serde_json::Value::Null);
    
    // Mobile context
    let mobile_ctx = use_mobile();
    let is_mobile = move || mobile_ctx.is_mobile.get();
    
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
            
            // View Switcher - dynamically switch between table/kanban/calendar/map
            <ViewSwitcher 
                entity_type=entity_type()
                on_view_change=move |view_def: ViewDefResponse| {
                    set_current_view_type.set(view_def.view_type.clone());
                    set_current_view_settings.set(view_def.settings.clone());
                }
            />
            
            // Error display
            {move || error.get().map(|e| view! {
                <div class="error-banner">{e}</div>
            })}
            
            // Loading state
            {move || loading.get().then(|| view! {
                <div class="loading">"Loading..."</div>
            })}
            
            // Dynamic view content based on current view type
            {move || (!loading.get()).then(|| {
                let view_type = current_view_type.get();
                let etype = entity_type();
                let settings = current_view_settings.get();
                
                match view_type.as_str() {
                    "kanban" => {
                        let group_by = settings.get("group_by_field")
                            .and_then(|v| v.as_str())
                            .unwrap_or("status")
                            .to_string();
                        let card_title = settings.get("card_title_field")
                            .and_then(|v| v.as_str())
                            .unwrap_or("title")
                            .to_string();
                        let kanban_config = KanbanConfig {
                            group_by_field: group_by,
                            card_title_field: card_title,
                            card_subtitle_field: settings.get("card_subtitle_field")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            card_fields: settings.get("card_fields")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                            column_order: settings.get("column_order")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                        };
                        view! {
                            <KanbanView 
                                entity_type=etype.clone()
                                config=kanban_config
                            />
                        }.into_view()
                    },
                    "calendar" => {
                        let calendar_config = CalendarConfig {
                            date_field: settings.get("date_field")
                                .and_then(|v| v.as_str())
                                .unwrap_or("scheduled_start")
                                .to_string(),
                            end_date_field: settings.get("end_date_field")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            title_field: settings.get("title_field")
                                .and_then(|v| v.as_str())
                                .unwrap_or("title")
                                .to_string(),
                            color_field: settings.get("color_field")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            color_map: None,
                        };
                        view! {
                            <CalendarView 
                                entity_type=etype.clone()
                                config=calendar_config
                            />
                        }.into_view()
                    },
                    "map" => {
                        let map_config = MapConfig {
                            lat_field: settings.get("lat_field")
                                .and_then(|v| v.as_str())
                                .unwrap_or("latitude")
                                .to_string(),
                            lng_field: settings.get("lng_field")
                                .and_then(|v| v.as_str())
                                .unwrap_or("longitude")
                                .to_string(),
                            popup_title_field: settings.get("popup_title_field")
                                .and_then(|v| v.as_str())
                                .unwrap_or("title")
                                .to_string(),
                            popup_fields: settings.get("popup_fields")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                            marker_color_field: settings.get("marker_color_field")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            default_center: None,
                            default_zoom: None,
                        };
                        view! {
                            <MapView 
                                entity_type=etype.clone()
                                config=map_config
                            />
                        }.into_view()
                    },
                    _ => {
                        // Default: Table view (or Mobile Card view)
                        let cols = list_columns();
                        
                        if is_mobile() {
                            // Mobile Card View
                            view! {
                                <div class="mobile-card-list">
                                    {move || {
                                        let etype = entity_type();
                                        let cols = list_columns();
                                        data.get().into_iter().map(|row| {
                                            let record_id = row.get("id")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or_default()
                                                .to_string();
                                            let etype_clone = etype.clone();
                                            let row_path = format!("/app/crm/entity/{}/{}", etype_clone, record_id);
                                            
                                            // Get title from first column
                                            let title = cols.first()
                                                .and_then(|f| row.get(&f.name))
                                                .map(|v| format_field_value(v, "text"))
                                                .unwrap_or_else(|| "Untitled".to_string());
                                            
                                            // Get subtitle from second column
                                            let subtitle = cols.get(1)
                                                .and_then(|f| row.get(&f.name))
                                                .map(|v| format_field_value(v, "text"))
                                                .unwrap_or_default();
                                            
                                            // Get phone/email for quick action
                                            let phone = row.get("phone")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            
                                            // Get status
                                            let status = row.get("status")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            
                                            view! {
                                                <a href=row_path class="mobile-card">
                                                    <div class="card-avatar">
                                                        <div class="avatar-placeholder">
                                                            {title.chars().next().unwrap_or('?').to_string()}
                                                        </div>
                                                    </div>
                                                    <div class="card-content">
                                                        <div class="card-title">{title}</div>
                                                        <div class="card-subtitle">{subtitle}</div>
                                                        {status.map(|s| view! {
                                                            <span class="card-status" data-status=s.to_lowercase()>{s}</span>
                                                        })}
                                                    </div>
                                                    <div class="card-actions">
                                                        {phone.map(|p| {
                                                            let phone_url = format!("tel:{}", p);
                                                            view! {
                                                                <span class="action-btn call-btn" on:click=move |e| {
                                                                    e.prevent_default();
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.open_with_url(&phone_url);
                                                                    }
                                                                }>
                                                                    "📞"
                                                                </span>
                                                            }
                                                        })}
                                                        <span class="card-arrow">"›"</span>
                                                    </div>
                                                </a>
                                            }
                                        }).collect_view()
                                    }}
                                    
                                    {move || data.get().is_empty().then(|| view! {
                                        <div class="empty-state">"No records found. Click + New to create one."</div>
                                    })}
                                </div>
                            }.into_view()
                        } else {
                            // Desktop Table View
                            view! {
                                <div class="table-container">
                                    <table class="data-table">
                                        <thead>
                                            <tr>
                                                {cols.iter().map(|f| {
                                                    view! { <th>{f.label.clone()}</th> }
                                                }).collect_view()}
                                                <th class="action-header">""</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || {
                                                let cols = list_columns();
                                                let etype = entity_type();
                                                data.get().into_iter().map(|row| {
                                                    let record_id = row.get("id")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or_default()
                                                        .to_string();
                                                    let etype_clone = etype.clone();
                                                    let row_path = format!("/app/crm/entity/{}/{}", etype_clone, record_id);
                                                    view! {
                                                        <tr class="clickable-row">
                                                            {cols.iter().map(|f| {
                                                                let value = row.get(&f.name)
                                                                    .map(|v| format_field_value(v, &f.get_field_type()))
                                                                    .unwrap_or_default();
                                                                view! { <td>{value}</td> }
                                                            }).collect_view()}
                                                            <td class="action-cell">
                                                                <a href=row_path class="row-link">"→"</a>
                                                            </td>
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
                            }.into_view()
                        }
                    }
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
                                    let field_type = field.get_field_type();
                                    let is_required = field.is_required;
                                    let placeholder = field.placeholder.clone().unwrap_or_default();
                                    let options = field.options.clone();
                                    
                                    view! {
                                        <div class="form-group">
                                            <label>
                                                {field_label}
                                                {is_required.then(|| " *")}
                                            </label>
                                            {render_input_field(field_name, field_type, is_required, placeholder, options, update_form_field.clone())}
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
    options: Option<serde_json::Value>,
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
        "select" => {
            // Extract options from FieldDef.options (expects array of {"value": "x", "label": "X"})
            let select_options: Vec<(String, String)> = options
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|opt| {
                    let value = opt.get("value")?.as_str()?.to_string();
                    let label = opt.get("label")?.as_str()?.to_string();
                    Some((value, label))
                })
                .collect();
            
            view! {
                <select required=is_required on:change=on_input>
                    <option value="">"-- Select --"</option>
                    {select_options.into_iter().map(|(value, label)| {
                        view! { <option value=value>{label}</option> }
                    }).collect_view()}
                </select>
            }.into_view()
        },
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
