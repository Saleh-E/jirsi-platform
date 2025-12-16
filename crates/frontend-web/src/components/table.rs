//! Smart Data Table Component with Inline Editing
//!
//! Features:
//! - Inline editing: double-click a cell to edit
//! - Auto-save on blur
//! - Status badges
//! - Row click navigation

use leptos::*;
use crate::api::FieldDef as ApiFieldDef;
use crate::models::ViewColumn;
use crate::api::{patch_json, API_BASE, TENANT_ID};

/// Smart Table with inline editing support
#[component]
pub fn SmartTable(
    /// Column definitions from view
    columns: Vec<ViewColumn>,
    /// Field definitions for type info
    fields: Vec<ApiFieldDef>,
    /// Data rows
    data: Vec<serde_json::Value>,
    /// Entity type for API calls
    entity_type: String,
    /// Callback when row is clicked (for navigation)
    #[prop(optional)] on_row_click: Option<Callback<serde_json::Value>>,
    /// Enable inline editing
    #[prop(optional, default = true)] editable: bool,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type);
    
    // Build field lookup
    let field_map: std::collections::HashMap<String, ApiFieldDef> = fields
        .into_iter()
        .map(|f| (f.name.clone(), f))
        .collect();
    let field_map_stored = store_value(field_map);

    let visible_columns: Vec<_> = columns
        .into_iter()
        .filter(|c| c.visible)
        .collect();
    let columns_stored = store_value(visible_columns);
    
    // Store data reactively so edits update the UI
    let (table_data, set_table_data) = create_signal(data);
    
    view! {
        <div class="smart-table-wrapper">
            <table class="smart-table">
                <thead>
                    <tr>
                        {columns_stored.get_value().iter().map(|col| {
                            let label = field_map_stored.get_value()
                                .get(&col.field)
                                .map(|f| f.label.clone())
                                .unwrap_or_else(|| col.field.clone());
                            view! {
                                <th class="smart-table-header">
                                    <span class="header-label">{label}</span>
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                        // Action column header when editable
                        {editable.then(|| view! {
                            <th class="smart-table-header action-header"></th>
                        })}
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || table_data.get()
                        key=|row| row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()
                        children=move |row| {
                            let row_id = row.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let row_for_click = row.clone();
                            let row_for_action = row.clone();
                            let on_row_click = on_row_click.clone();
                            let on_row_click_action = on_row_click.clone();
                            let entity = entity_type_stored.get_value();
                            
                            view! {
                                <tr class="smart-table-row">
                                    {columns_stored.get_value().iter().map(|col| {
                                        let field_name = col.field.clone();
                                        let value = row.get(&col.field)
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null);
                                        let field_def = field_map_stored.get_value().get(&col.field).cloned();
                                        let row_id_cell = row_id.clone();
                                        let entity_cell = entity.clone();
                                        let on_click = on_row_click.clone();
                                        let row_data = row_for_click.clone();
                                        
                                        view! {
                                            <SmartTableCell
                                                value=value
                                                field_def=field_def
                                                field_name=field_name
                                                row_id=row_id_cell
                                                entity_type=entity_cell
                                                editable=editable
                                                set_table_data=set_table_data
                                                on_click=Callback::new(move |_: ()| {
                                                    if let Some(ref cb) = on_click {
                                                        cb.call(row_data.clone());
                                                    }
                                                })
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                    // Action column with arrow button when editable
                                    {editable.then(|| {
                                        let row_for_nav = row_for_action.clone();
                                        let on_nav = on_row_click_action.clone();
                                        view! {
                                            <td class="smart-table-cell action-cell">
                                                <button 
                                                    class="row-action-btn" 
                                                    on:click=move |_| {
                                                        if let Some(ref cb) = on_nav {
                                                            cb.call(row_for_nav.clone());
                                                        }
                                                    }
                                                >
                                                    "→"
                                                </button>
                                            </td>
                                        }
                                    })}
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}

/// Individual cell with inline editing
#[component]
fn SmartTableCell(
    value: serde_json::Value,
    field_def: Option<ApiFieldDef>,
    field_name: String,
    row_id: String,
    entity_type: String,
    editable: bool,
    set_table_data: WriteSignal<Vec<serde_json::Value>>,
    #[prop(into)] on_click: Callback<()>,
) -> impl IntoView {
    let (is_editing, set_is_editing) = create_signal(false);
    let (local_value, set_local_value) = create_signal(value.clone());
    let original_value = store_value(value.clone());
    
    let field_type = field_def.as_ref()
        .map(|f| f.get_field_type())
        .unwrap_or_else(|| "text".to_string());
    
    let field_name_stored = store_value(field_name.clone());
    let row_id_stored = store_value(row_id.clone());
    let entity_type_stored = store_value(entity_type.clone());
    
    // Handle double-click to edit - must stop propagation to prevent row click
    let handle_dbl_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        ev.prevent_default();
        if editable {
            set_is_editing.set(true);
        }
    };
    
    // Handle single click - if editable, don't navigate (let dblclick handle editing)
    // If not editable, navigate on click
    let handle_click = move |ev: web_sys::MouseEvent| {
        if !editable && !is_editing.get() {
            ev.stop_propagation();
            on_click.call(());
        }
        // When editable, single click does nothing - user must double-click to edit
    };
    
    // Handle blur to save
    let handle_blur = move |_| {
        set_is_editing.set(false);
        let new_val = local_value.get();
        let old_val = original_value.get_value();
        
        // Only save if value changed
        if new_val != old_val {
            let field = field_name_stored.get_value();
            let rid = row_id_stored.get_value();
            let entity = entity_type_stored.get_value();
            
            // Update local table data optimistically
            let field_clone = field.clone();
            let new_val_clone = new_val.clone();
            set_table_data.update(|rows| {
                if let Some(row) = rows.iter_mut().find(|r| {
                    r.get("id").and_then(|v| v.as_str()) == Some(&rid)
                }) {
                    if let Some(obj) = row.as_object_mut() {
                        obj.insert(field_clone.clone(), new_val_clone.clone());
                    }
                }
            });
            
            // API call to persist
            spawn_local(async move {
                let url = format!("{}/entities/{}/records/{}?tenant_id={}", 
                    API_BASE, entity, rid, TENANT_ID);
                
                let mut payload = std::collections::HashMap::new();
                payload.insert(field, new_val);
                
                // Ignore result - optimistic update already applied
                let _: Result<serde_json::Value, _> = patch_json(&url, &payload).await;
            });
        }
    };
    
    // Handle Enter key
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            set_is_editing.set(false);
            
            let new_val = local_value.get();
            let old_val = original_value.get_value();
            
            if new_val != old_val {
                let field = field_name_stored.get_value();
                let rid = row_id_stored.get_value();
                let entity = entity_type_stored.get_value();
                
                let field_clone = field.clone();
                let new_val_clone = new_val.clone();
                set_table_data.update(|rows| {
                    if let Some(row) = rows.iter_mut().find(|r| {
                        r.get("id").and_then(|v| v.as_str()) == Some(&rid)
                    }) {
                        if let Some(obj) = row.as_object_mut() {
                            obj.insert(field_clone.clone(), new_val_clone.clone());
                        }
                    }
                });
                
                spawn_local(async move {
                    let url = format!("{}/entities/{}/records/{}?tenant_id={}", 
                        API_BASE, entity, rid, TENANT_ID);
                    
                    let mut payload = std::collections::HashMap::new();
                    payload.insert(field, new_val);
                    
                    let _: Result<serde_json::Value, _> = patch_json(&url, &payload).await;
                });
            }
        } else if ev.key() == "Escape" {
            set_local_value.set(original_value.get_value());
            set_is_editing.set(false);
        }
    };
    
    view! {
        <td 
            class="smart-table-cell"
            class:editing=move || is_editing.get()
            on:click=handle_click
            on:dblclick=handle_dbl_click
        >
            {move || {
                if is_editing.get() {
                    // Edit mode
                    match field_type.to_lowercase().as_str() {
                        "text" | "email" | "phone" | "url" => {
                            let current = local_value.get().as_str().unwrap_or("").to_string();
                            view! {
                                <input
                                    type="text"
                                    class="cell-input"
                                    value=current
                                    autofocus=true
                                    on:input=move |ev| {
                                        set_local_value.set(serde_json::Value::String(event_target_value(&ev)));
                                    }
                                    on:blur=handle_blur
                                    on:keydown=handle_keydown
                                />
                            }.into_view()
                        }
                        "number" | "integer" | "decimal" | "money" => {
                            let current = local_value.get().as_f64().unwrap_or(0.0).to_string();
                            view! {
                                <input
                                    type="number"
                                    class="cell-input"
                                    value=current
                                    autofocus=true
                                    on:input=move |ev| {
                                        if let Ok(n) = event_target_value(&ev).parse::<f64>() {
                                            set_local_value.set(serde_json::json!(n));
                                        }
                                    }
                                    on:blur=handle_blur
                                    on:keydown=handle_keydown
                                />
                            }.into_view()
                        }
                        _ => {
                            // Default text input
                            let current = local_value.get().as_str().unwrap_or("").to_string();
                            view! {
                                <input
                                    type="text"
                                    class="cell-input"
                                    value=current
                                    autofocus=true
                                    on:input=move |ev| {
                                        set_local_value.set(serde_json::Value::String(event_target_value(&ev)));
                                    }
                                    on:blur=handle_blur
                                    on:keydown=handle_keydown
                                />
                            }.into_view()
                        }
                    }
                } else {
                    // Display mode
                    render_cell_display(&field_type, &local_value.get())
                }
            }}
        </td>
    }
}

/// Render cell display value with appropriate formatting
fn render_cell_display(field_type: &str, value: &serde_json::Value) -> View {
    match field_type.to_lowercase().as_str() {
        "status" | "select" => {
            let text = value.as_str().unwrap_or("");
            let status_class = get_status_class(text);
            view! {
                <span class=format!("status-badge {}", status_class)>
                    {text.to_string()}
                </span>
            }.into_view()
        }
        "boolean" => {
            let checked = value.as_bool().unwrap_or(false);
            view! {
                <span class="bool-display">
                    {if checked { "✓" } else { "—" }}
                </span>
            }.into_view()
        }
        "money" | "currency" => {
            let amount = value.as_f64().unwrap_or(0.0);
            view! {
                <span class="money-display">
                    {format!("${:.2}", amount)}
                </span>
            }.into_view()
        }
        "date" => {
            let date_str = value.as_str().unwrap_or("");
            view! {
                <span class="date-display">{date_str.to_string()}</span>
            }.into_view()
        }
        "email" => {
            let email = value.as_str().unwrap_or("");
            view! {
                <a class="email-link" href=format!("mailto:{}", email)>
                    {email.to_string()}
                </a>
            }.into_view()
        }
        "phone" => {
            let phone = value.as_str().unwrap_or("");
            view! {
                <a class="phone-link" href=format!("tel:{}", phone)>
                    {phone.to_string()}
                </a>
            }.into_view()
        }
        _ => {
            let text = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
                serde_json::Value::Null => "—".to_string(),
                _ => value.to_string(),
            };
            view! {
                <span class="cell-text">{text}</span>
            }.into_view()
        }
    }
}

/// Get CSS class for status badge
fn get_status_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "new" | "open" | "active" => "status-new",
        "in_progress" | "pending" | "working" => "status-progress",
        "won" | "completed" | "closed" | "success" => "status-won",
        "lost" | "cancelled" | "failed" => "status-lost",
        "qualified" | "contacted" => "status-qualified",
        _ => "status-default",
    }
}

// Keep the old DataTable for backwards compatibility
use crate::models::FieldDef;

/// Generic data table based on ViewDef columns (legacy - use SmartTable)
#[component]
pub fn DataTable(
    columns: Vec<ViewColumn>,
    fields: Vec<FieldDef>,
    data: Vec<serde_json::Value>,
    #[prop(optional)] _on_row_click: Option<Callback<serde_json::Value>>,
) -> impl IntoView {
    // Build field lookup
    let field_map: std::collections::HashMap<String, FieldDef> = fields
        .into_iter()
        .map(|f| (f.name.clone(), f))
        .collect();

    let visible_columns: Vec<_> = columns
        .into_iter()
        .filter(|c| c.visible)
        .collect();

    view! {
        <div class="data-table-wrapper">
            <table class="data-table">
                <thead>
                    <tr>
                        {visible_columns.iter().map(|col| {
                            let label = field_map
                                .get(&col.field)
                                .map(|f| f.label.clone())
                                .unwrap_or_else(|| col.field.clone());
                            view! {
                                <th class="table-header">{label}</th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody>
                    {data.iter().map(|row| {
                        view! {
                            <tr class="table-row">
                                {visible_columns.iter().map(|col| {
                                    let value = row.get(&col.field)
                                        .cloned()
                                        .unwrap_or(serde_json::Value::Null);
                                    view! {
                                        <td class="table-cell">{value.to_string()}</td>
                                    }
                                }).collect::<Vec<_>>()}
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
