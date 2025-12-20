//! Generic Entity List Page - Metadata-driven
//! 
//! This component renders entity lists dynamically using FieldDefs metadata.
//! Columns and form fields are NOT hardcoded per entity type.
//! Supports multiple view types via ViewSwitcher.
//! Persists view selection in URL query params (?view=kanban)

use leptos::*;
use leptos_router::*;
use crate::api::{
    fetch_field_defs, fetch_entity_list, FieldDef, ViewColumn, ViewType, fetch_default_view
};
use crate::components::view_switcher::{ViewSwitcher, ViewDefResponse};
use crate::components::kanban::{KanbanView, KanbanConfig};
use crate::components::calendar::{CalendarView, CalendarConfig};
use crate::components::map::{MapView, MapConfig};
use crate::components::table::SmartTable;
use crate::components::create_modal::CreateModal;
use crate::context::mobile::use_mobile;


/// Main entity list page component
#[component]
pub fn EntityListPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = use_navigate();
    
    let entity_type = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    
    // Signals
    let (fields, set_fields) = create_signal(Vec::<FieldDef>::new());
    let (data, set_data) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(true);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    
    // View switching state
    let (current_view_type, set_current_view_type) = create_signal("table".to_string());
    let (current_view_settings, set_current_view_settings) = create_signal(serde_json::Value::Null);
    
    // NEW: Columns state driven by ViewDef
    let (current_columns, set_current_columns) = create_signal(Vec::<ViewColumn>::new());
    
    // Sync view type from URL query param (reactive)
    create_effect(move |_| {
        let view_from_url = query.with(|q| {
            q.get("view").cloned().unwrap_or_else(|| "table".to_string())
        });
        set_current_view_type.set(view_from_url);
    });
    
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
    
    // Get list columns (Priority: ViewDef columns -> Fallback: show_in_list fields)
    let list_columns = move || {
        let view_cols = current_columns.get();
        if !view_cols.is_empty() {
            return view_cols;
        }
        
        // Fallback: Generate generic columns from fields marked show_in_list
        fields.get()
            .into_iter()
            .filter(|f| f.show_in_list)
            .map(|f| ViewColumn {
                field: f.name,
                width: None,
                visible: true,
                sort_order: f.sort_order,
            })
            .collect::<Vec<_>>()
    };
    
    // Callback when record is created via CreateModal
    let entity_for_refresh = entity_type.clone();
    let on_record_created = move |_record: crate::components::create_modal::CreatedRecord| {
        let etype = entity_for_refresh();
        spawn_local(async move {
            if let Ok(response) = fetch_entity_list(&etype).await {
                set_data.set(response.data);
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
                <button class="btn btn-primary" on:click=move |_| set_show_create_modal.set(true)>
                    "+ New"
                </button>
            </header>
            
            // View Switcher - dynamically switch between table/kanban/calendar/map
            <ViewSwitcher 
                entity_type=entity_type()
                on_view_change=move |view_def: ViewDefResponse| {
                    let new_view = view_def.view_type.clone();
                    set_current_view_type.set(new_view.clone());
                    set_current_view_settings.set(view_def.settings.clone());
                    set_current_columns.set(view_def.columns.clone());
                    
                    // Update URL query param
                    let etype = entity_type();
                    let new_url = format!("/app/crm/entity/{}?view={}", etype, new_view);
                    navigate(&new_url, NavigateOptions {
                        replace: true,
                        ..Default::default()
                    });
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
                        
                        if is_mobile() {
                            // Mobile Card View
                            view! {
                                <div class="mobile-card-list">
                                    {move || {
                                        let etype = entity_type();
                                        // Mobile view still needs to know which fields to show
                                        // Uses same cols logic but renders differently
                                        let cols_ref = list_columns(); 
                                        data.get().into_iter().map(|row| {
                                            let record_id = row.get("id")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or_default()
                                                .to_string();
                                            let etype_clone = etype.clone();
                                            let row_path = format!("/app/crm/entity/{}/{}", etype_clone, record_id);
                                            
                                            // Get title from first column
                                            let title = cols_ref.first()
                                                .and_then(|c| row.get(&c.field))
                                                .map(|v| format_field_value(v, "text"))
                                                .unwrap_or_else(|| "Untitled".to_string());
                                            
                                            // Get subtitle from second column
                                            let subtitle = cols_ref.get(1)
                                                .and_then(|c| row.get(&c.field))
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
                            // Desktop Table View with SmartTable (inline editing)
                            let etype = entity_type();
                            // Direct pass-through of ViewColumns
                            let view_cols = list_columns();
                            let api_fields = fields.get();
                            let navigate = use_navigate();
                            
                            view! {
                                <SmartTable
                                    columns=view_cols
                                    fields=api_fields
                                    data=data.get()
                                    entity_type=etype.clone()
                                    on_row_click=Callback::new(move |row: serde_json::Value| {
                                        let record_id = row.get("id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or_default()
                                            .to_string();
                                        let path = format!("/app/crm/entity/{}/{}", etype.clone(), record_id);
                                        navigate(&path, Default::default());
                                    })
                                    editable=true
                                />
                                
                                {move || data.get().is_empty().then(|| view! {
                                    <div class="empty-state">"No records found. Click + New to create one."</div>
                                })}
                            }.into_view()
                        }
                    }
                }
            })}
            
            // Create Modal with SmartSelect components
            {move || show_create_modal.get().then(|| {
                let etype = entity_type();
                let elabel = match etype.as_str() {
                    "contact" => "Contact".to_string(),
                    "company" => "Company".to_string(),
                    "deal" => "Deal".to_string(),
                    "task" => "Task".to_string(),
                    "property" => "Property".to_string(),
                    "listing" => "Listing".to_string(),
                    "viewing" => "Viewing".to_string(),
                    "offer" => "Offer".to_string(),
                    _ => etype[0..1].to_uppercase() + &etype[1..],
                };
                view! {
                    <CreateModal
                        entity_type=etype
                        entity_label=elabel
                        on_close=Callback::new(move |_| set_show_create_modal.set(false))
                        on_created=Callback::new(on_record_created.clone())
                    />
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
