//! Editable Table Component - Inline editing with auto-save
//!
//! Click any cell to edit, changes save on blur via PATCH API

use leptos::*;
use wasm_bindgen::JsCast;
use crate::api::{FieldDef, patch_json, API_BASE, TENANT_ID};
use crate::components::field_renderer::EditableFieldValue;

/// Editable data table with inline editing
#[component]
pub fn EditableTable(
    /// Entity type for API calls
    entity_type: String,
    /// Column definitions
    columns: Vec<FieldDef>,
    /// Row data
    data: RwSignal<Vec<serde_json::Value>>,
    /// Density: "compact" | "comfortable" | "spacious"
    #[prop(default = "comfortable".to_string())]
    density: String,
    /// Callback when row is clicked (for navigation)
    #[prop(optional, into)]
    on_row_click: Option<Callback<String>>,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type);
    
    // Handle cell save
    let save_cell = move |record_id: String, field_name: String, new_value: serde_json::Value| {
        let etype = entity_type_stored.get_value();
        spawn_local(async move {
            let mut patch_body = serde_json::Map::new();
            patch_body.insert(field_name.clone(), new_value.clone());
            
            let url = format!("{}/entity/{}/{}?tenant_id={}", API_BASE, etype, record_id, TENANT_ID);
            match patch_json::<serde_json::Value, serde_json::Value>(&url, &serde_json::Value::Object(patch_body)).await {
                Ok(_) => {
                    logging::log!("Saved {} = {:?}", field_name, new_value);
                }
                Err(e) => {
                    logging::error!("Failed to save: {}", e);
                    // TODO: Show error toast
                }
            }
        });
    };
    
    let density_class = format!("density-{}", density);
    
    view! {
        <div class=format!("editable-table-wrapper {}", density_class)>
            <table class="editable-table">
                <thead>
                    <tr>
                        {columns.iter().map(|col| {
                            view! {
                                <th class="table-header" title=col.name.clone()>
                                    <span>{col.label.clone()}</span>
                                </th>
                            }
                        }).collect_view()}
                        <th class="action-header"></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cols = columns.clone();
                        let on_click = on_row_click.clone();
                        
                        data.get().into_iter().map(|row| {
                            let record_id = row.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            
                            let row_id = record_id.clone();
                            let nav_id = record_id.clone();
                            
                            view! {
                                <tr class="table-row">
                                    {cols.iter().map(|col| {
                                        let field = col.clone();
                                        let value = row.get(&col.name)
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null);
                                        let rid = row_id.clone();
                                        let fname = col.name.clone();
                                        
                                        // Determine if field is editable (not readonly, not id)
                                        let is_editable = !col.is_readonly && col.name != "id" && col.name != "created_at" && col.name != "updated_at";
                                        
                                        view! {
                                            <td class="table-cell" class:editable=is_editable>
                                                {if is_editable {
                                                    view! {
                                                        <EditableFieldValue 
                                                            field=field
                                                            value=value
                                                            on_change=Callback::new(move |new_val: serde_json::Value| {
                                                                save_cell(rid.clone(), fname.clone(), new_val);
                                                            })
                                                        />
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <span class="readonly-value">
                                                            {format_display_value(&value, &col.get_field_type())}
                                                        </span>
                                                    }.into_view()
                                                }}
                                            </td>
                                        }
                                    }).collect_view()}
                                    <td class="action-cell">
                                        {on_click.map(|cb| {
                                            let id = nav_id.clone();
                                            view! {
                                                <button 
                                                    class="row-action-btn"
                                                    on:click=move |_| cb.call(id.clone())
                                                    title="Open"
                                                >
                                                    "→"
                                                </button>
                                            }
                                        })}
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
            </table>
            
            {move || data.get().is_empty().then(|| view! {
                <div class="empty-state">
                    <span class="empty-icon">"📋"</span>
                    <h3>"No records found"</h3>
                    <p>"Create your first record using the + New button"</p>
                </div>
            })}
        </div>
    }
}

fn format_display_value(value: &serde_json::Value, field_type: &str) -> String {
    match value {
        serde_json::Value::Null => "—".to_string(),
        serde_json::Value::String(s) => {
            if field_type == "date" && s.len() >= 10 {
                s[..10].to_string()
            } else {
                s.clone()
            }
        }
        serde_json::Value::Number(n) => {
            if field_type == "currency" {
                format!("${:.2}", n.as_f64().unwrap_or(0.0))
            } else {
                n.to_string()
            }
        }
        serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        serde_json::Value::Array(arr) => format!("{} items", arr.len()),
        serde_json::Value::Object(_) => "[object]".to_string(),
    }
}
