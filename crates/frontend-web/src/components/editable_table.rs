//! Editable Table Component - Inline editing with auto-save
//!
//! Click any cell to edit, changes save on blur via PATCH API


use leptos::*;
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
    
    let padding_y = match density.as_str() {
        "compact" => "py-1",
        "spacious" => "py-5",
        _ => "py-3",
    };
    
    view! {
        <div class="w-full overflow-x-auto rounded-xl border border-white/10 bg-surface/30 backdrop-blur-md">
            <table class="w-full border-collapse text-left">
                <thead>
                    <tr>
                        {columns.iter().map(|col| {
                            view! {
                                <th class="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500 border-b border-white/10 whitespace-nowrap" title=col.name.clone()>
                                    <span>{col.label.clone()}</span>
                                </th>
                            }
                        }).collect_view()}
                        <th class="px-4 py-3 border-b border-white/10 w-12"></th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-white/5">
                    {move || {
                        let cols = columns.clone();
                        let on_click = on_row_click.clone();
                        let py_class = padding_y.clone();
                        
                        data.get().into_iter().map(|row| {
                            let record_id = row.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            
                            let row_id = record_id.clone();
                            let nav_id = record_id.clone();
                            let py = py_class.clone();
                            
                            view! {
                                <tr class="group transition-colors hover:bg-white/5">
                                    {cols.iter().map(|col| {
                                        let field = col.clone();
                                        let value = row.get(&col.name)
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null);
                                        let rid = row_id.clone();
                                        let fname = col.name.clone();
                                        let py_cell = py.clone();
                                        
                                        // Determine if field is editable (not readonly, not id)
                                        let is_editable = !col.is_readonly && col.name != "id" && col.name != "created_at" && col.name != "updated_at";
                                        
                                        view! {
                                            <td class=format!("px-6 {} text-sm text-slate-300 whitespace-nowrap transition-colors", py_cell) class:bg-white_5=is_editable class:hover:bg-white_10=is_editable>
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
                                                        <span class="opacity-70">
                                                            {format_display_value(&value, &col.get_field_type())}
                                                        </span>
                                                    }.into_view()
                                                }}
                                            </td>
                                        }
                                    }).collect_view()}
                                    <td class="px-4 py-3 whitespace-nowrap text-right">
                                        {on_click.map(|cb| {
                                            let id = nav_id.clone();
                                            view! {
                                                <button 
                                                    class="w-8 h-8 inline-flex items-center justify-center rounded-full text-slate-400 hover:text-white hover:bg-white/10 transition-colors opacity-0 group-hover:opacity-100"
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
                <div class="flex flex-col items-center justify-center py-16 text-slate-500">
                    <span class="text-4xl mb-4 opacity-50">"📋"</span>
                    <h3 class="text-lg font-medium text-slate-400 mb-2">"No records found"</h3>
                    <p class="text-sm">"Create your first record using the + New button"</p>
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
