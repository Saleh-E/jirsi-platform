//! Smart Data Table Component with Inline Editing
//!
//! Features:
//! - Inline editing: double-click a cell to edit
//! - Auto-save on blur
//! - Status badges
//! - Row click navigation
//! - SmartSelect for searchable dropdowns

use leptos::*;
use crate::api::FieldDef as ApiFieldDef;
use crate::api::ViewColumn;
use crate::api::{put_json, add_field_option, delete_field_option, API_BASE, TENANT_ID};
use crate::components::smart_select::{SmartSelect, SelectOption};
use crate::components::async_entity_select::{AsyncEntitySelect, AsyncEntityLabel};

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
    /// Enable row selection checkboxes
    #[prop(optional, default = false)] selectable: bool,
    /// Callback when selection changes - returns count of selected items
    #[prop(optional)] on_selection_change: Option<Callback<usize>>,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type);
    
    // Build field lookup
    let field_map: std::collections::HashMap<String, ApiFieldDef> = fields.clone()
        .into_iter()
        .map(|f| (f.name.clone(), f))
        .collect();
    let field_map_stored = store_value(field_map);
    
    // Track selected rows
    let (selected_ids, set_selected_ids) = create_signal::<std::collections::HashSet<String>>(std::collections::HashSet::new());
    
    // Create SHARED options signals for each select/status field
    // This way all cells in the same column share the same options
    let shared_field_options: std::collections::HashMap<String, (ReadSignal<Vec<SelectOption>>, WriteSignal<Vec<SelectOption>>)> = 
        fields.iter()
            .filter(|f| {
                let ft = f.get_field_type().to_lowercase();
                ft == "select" || ft == "status"
            })
            .map(|f| {
                let options = f.get_options().into_iter()
                    .map(|(v, l)| SelectOption::new(v, l))
                    .collect::<Vec<_>>();
                // Debug: Log the options being loaded for each field
                web_sys::console::log_1(&format!(
                    "🔧 SmartTable: Loading {} options for field '{}': {:?}",
                    options.len(),
                    f.name,
                    options.iter().map(|o| o.value.clone()).collect::<Vec<_>>()
                ).into());
                (f.name.clone(), create_signal(options))
            })
            .collect();
    let shared_options_stored = store_value(shared_field_options);

    let visible_columns: Vec<_> = columns
        .into_iter()
        .filter(|c| c.visible)
        .collect();
    let columns_stored = store_value(visible_columns);
    
    // Store data reactively so edits update the UI
    let (table_data, set_table_data) = create_signal(data.clone());
    let all_ids: Vec<String> = data.iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();
    let all_ids_stored = store_value(all_ids);
    
    // Global editing cell: (row_id, field_name) - only one cell can edit at a time
    let (editing_cell, set_editing_cell) = create_signal::<Option<(String, String)>>(None);
    
    // Notify parent of selection changes
    let on_selection_change_stored = store_value(on_selection_change);
    create_effect(move |_| {
        let count = selected_ids.get().len();
        if let Some(cb) = on_selection_change_stored.get_value() {
            cb.call(count);
        }
    });
    
    // Toggle all checkbox
    let toggle_all = move |_| {
        let current = selected_ids.get();
        let all = all_ids_stored.get_value();
        if current.len() == all.len() {
            set_selected_ids.set(std::collections::HashSet::new());
        } else {
            set_selected_ids.set(all.into_iter().collect());
        }
    };
    
    // Toggle single row
    let toggle_row = move |row_id: String| {
        set_selected_ids.update(|ids| {
            if ids.contains(&row_id) {
                ids.remove(&row_id);
            } else {
                ids.insert(row_id);
            }
        });
    };
    
    view! {
        <div class="w-full overflow-x-auto rounded-xl border border-white/10 bg-surface/30 backdrop-blur-md">
            <table class="w-full border-collapse text-left">
                <thead>
                    <tr>
                        // Checkbox header for bulk select
                        {selectable.then(|| {
                            let all_count = all_ids_stored.get_value().len();
                            view! {
                                <th class="px-4 py-3 text-xs font-bold uppercase tracking-wider text-slate-500 border-b border-white/10 w-10">
                                    <input type="checkbox" class="rounded border-slate-600 bg-white/5 text-indigo-500 focus:ring-indigo-500 focus:ring-offset-0 cursor-pointer" checked={move || selected_ids.get().len() == all_count && all_count > 0} on:change=toggle_all />
                                </th>
                            }
                        })}
                        {columns_stored.get_value().iter().map(|col| {
                            let label = field_map_stored.get_value()
                                .get(&col.field)
                                .map(|f| f.label.clone())
                                .unwrap_or_else(|| col.field.clone());
                            view! {
                                <th class="px-6 py-3 text-xs font-bold uppercase tracking-wider text-slate-500 border-b border-white/10 whitespace-nowrap">
                                    <span class="flex items-center gap-2">{label}</span>
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                        // Action column header when editable
                        {editable.then(|| view! {
                            <th class="px-4 py-3 border-b border-white/10 w-12"></th>
                        })}
                    </tr>
                </thead>
                <tbody class="divide-y divide-white/5">
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
                                <tr class="group transition-colors hover:bg-white/5">
                                    // Checkbox cell for row selection
                                    {selectable.then(|| {
                                        let rid = row_id.clone();
                                        let rid_check = row_id.clone();
                                        view! {
                                            <td class="px-4 py-3 whitespace-nowrap">
                                                <input type="checkbox" class="rounded border-slate-600 bg-white/5 text-indigo-500 focus:ring-indigo-500 focus:ring-offset-0 cursor-pointer" checked=move || selected_ids.get().contains(&rid_check) on:change=move |_| toggle_row(rid.clone()) on:click=|ev: web_sys::MouseEvent| ev.stop_propagation() />
                                            </td>
                                        }
                                    })}
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
                                        
                                        // Get shared options for this field (if it's a select/status field)
                                        let field_shared_options = shared_options_stored.get_value()
                                            .get(&col.field)
                                            .cloned();
                                        
                                        view! {
                                            <SmartTableCell
                                                value=value
                                                field_def=field_def
                                                field_name=field_name.clone()
                                                row_id=row_id_cell.clone()
                                                entity_type=entity_cell
                                                editable=editable
                                                set_table_data=set_table_data
                                                editing_cell=editing_cell
                                                set_editing_cell=set_editing_cell
                                                shared_options=field_shared_options
                                                _on_click=Callback::new(move |_: ()| {
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
                                            <td class="px-4 py-3 whitespace-nowrap text-right">
                                                <button 
                                                    class="w-8 h-8 inline-flex items-center justify-center rounded-full text-slate-400 hover:text-white hover:bg-white/10 transition-colors opacity-0 group-hover:opacity-100" 
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
    editing_cell: ReadSignal<Option<(String, String)>>,
    set_editing_cell: WriteSignal<Option<(String, String)>>,
    #[prop(into)] _on_click: Callback<()>,
    /// Shared options signal for this field (shared across all rows)
    /// Pass None for non-select fields
    shared_options: Option<(ReadSignal<Vec<SelectOption>>, WriteSignal<Vec<SelectOption>>)>,
) -> impl IntoView {
    // Use store_value to avoid re-renders when value changes during editing
    let local_value = store_value(value.clone());
    let original_value = store_value(value.clone());
    
    let field_type = field_def.as_ref()
        .map(|f| f.get_field_type())
        .unwrap_or_else(|| "text".to_string());
    
    let field_name_stored = store_value(field_name.clone());
    let row_id_stored = store_value(row_id.clone());
    let entity_type_stored = store_value(entity_type.clone());

    // Extract target entity for Link fields
    let target_entity = field_def.as_ref()
        .and_then(|f| {
             // field_type can be {"Link": {"target_entity": "property"}}
             if let Some(obj) = f.field_type.as_object() {
                 if let Some(link_obj) = obj.get("Link").and_then(|v| v.as_object()) {
                     return link_obj.get("target_entity").and_then(|v| v.as_str()).map(|s| s.to_string());
                 }
             }
             None
        })
        .unwrap_or_else(|| "entity".to_string());
    let target_entity_stored = store_value(target_entity);
    
    // Use SHARED options from parent if provided, otherwise fall back to field_def options
    // This ensures all cells in the same column share the same options
    let (get_options, set_shared_options) = shared_options.unwrap_or_else(|| {
        // Fallback: create per-cell options (shouldn't happen for select/status fields)
        let field_options: Vec<SelectOption> = field_def.as_ref()
            .map(|f| f.get_options().into_iter().map(|(v, l)| SelectOption::new(v, l)).collect())
            .unwrap_or_default();
        create_signal(field_options)
    });
    
    // Store field ID for add_field_option API call
    let field_id_stored = store_value(field_def.as_ref().map(|f| f.id.clone()).unwrap_or_default());
    
    // Cell ID for global editing comparison
    let cell_id = (row_id.clone(), field_name.clone());
    let cell_id_stored = store_value(cell_id.clone());
    
    // Check if THIS cell is the one being edited
    let is_editing = move || {
        editing_cell.get() == Some(cell_id_stored.get_value())
    };
    
    // Track previous editing state to detect when this cell stops being edited
    let (was_editing, set_was_editing) = create_signal(false);
    
    // Auto-save when this cell stops being the editing cell
    create_effect(move |_| {
        let currently_editing = is_editing();
        let previously_editing = was_editing.get();
        
        if previously_editing && !currently_editing {
            // This cell just stopped being edited - save it
            let new_val = local_value.get_value();
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
                    let url = format!("{}/entities/{}/{}?tenant_id={}", 
                        API_BASE, entity, rid, TENANT_ID);
                    let mut map = serde_json::Map::new();
                    map.insert(field, new_val);
                    let payload = serde_json::Value::Object(map);
                    let _: Result<serde_json::Value, _> = put_json(&url, &payload).await;
                });
                
                original_value.set_value(new_val_clone);
            }
        }
        
        set_was_editing.set(currently_editing);
    });
    
    // Save current cell value and persist to API
    let save_cell = move || {
        let new_val = local_value.get_value();
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
                let url = format!("{}/entities/{}/{}?tenant_id={}", 
                    API_BASE, entity, rid, TENANT_ID);
                let mut map = serde_json::Map::new();
                map.insert(field, new_val);
                let payload = serde_json::Value::Object(map);
                let _: Result<serde_json::Value, _> = put_json(&url, &payload).await;
            });
            
            // Update original to new value
            original_value.set_value(new_val_clone);
        }
    };
    
    // Handle single click to edit - effect will auto-save previous cell
    let handle_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        if editable {
            // Just set this cell as editing - the effect on the previous cell
            // will detect it stopped being edited and auto-save
            set_editing_cell.set(Some(cell_id_stored.get_value()));
        }
    };
    
    // Handle confirm button click
    let handle_confirm = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        save_cell();
        set_editing_cell.set(None);
    };
    
    // Handle blur - save and close
    let handle_blur = move |_| {
        save_cell();
        set_editing_cell.set(None);
    };
    
    // Handle Enter key - save and close
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            save_cell();
            set_editing_cell.set(None);
        } else if ev.key() == "Escape" {
            // Revert to original value
            local_value.set_value(original_value.get_value());
            set_editing_cell.set(None);
        }
    };
    
    view! {
        <td 
            class="px-6 py-4 text-sm text-slate-300 whitespace-nowrap transition-colors"
            class:bg-white_5=is_editing
            class:ring-2=is_editing
            class:ring-indigo-500_50=is_editing
            class:ring-inset=is_editing
            on:click=handle_click
        >
            {move || {
                let ft = field_type.clone();
                let opts = get_options.get();
                
                if is_editing() {
                    // Edit mode with confirm button
                    match ft.to_lowercase().as_str() {
                        "select" | "status" => {
                            // SmartSelect for searchable select/status fields - use stored SelectOptions with value/label
                            let current = local_value.get_value().as_str().unwrap_or("").to_string();
                            let select_options = opts.clone(); // Already Vec<SelectOption>
                            
                            // Handle selection change and auto-save
                            let handle_select_change = move |val: String| {
                                local_value.set_value(serde_json::Value::String(val));
                                save_cell();
                                set_editing_cell.set(None);
                            };
                            
                            // Handle inline creation - when user types a new value and clicks "+ Add 'value'"
                            let handle_create_value = move |new_value: String| {
                                // First, add the new option to the local options so it appears immediately
                                let new_option = SelectOption::new(new_value.clone(), new_value.clone());
                                set_shared_options.update(|opts| {
                                    // Check if not already exists
                                    if !opts.iter().any(|o| o.value == new_value) {
                                        opts.push(new_option);
                                    }
                                });
                                
                                // Save the new value to the entity
                                local_value.set_value(serde_json::Value::String(new_value.clone()));
                                save_cell();
                                set_editing_cell.set(None);
                                
                                // Also persist the new option to the field definition via API
                                let entity = entity_type_stored.get_value();
                                let field_id = field_id_stored.get_value();
                                let value_clone = new_value.clone();
                                
                                // Debug: Log field_id to verify it's available
                                web_sys::console::log_1(&format!("DEBUG: field_id='{}', entity='{}', new_value='{}'", field_id, entity, value_clone).into());
                                
                                if !field_id.is_empty() {
                                    spawn_local(async move {
                                        web_sys::console::log_1(&format!("Calling add_field_option API for field {}", field_id).into());
                                        match add_field_option(&entity, &field_id, &value_clone, Some(&value_clone)).await {
                                            Ok(_) => {
                                                web_sys::console::log_1(&format!("✅ Added option '{}' to field {}", value_clone, field_id).into());
                                            }
                                            Err(e) => {
                                                web_sys::console::error_1(&format!("❌ Failed to persist option: {}", e).into());
                                            }
                                        }
                                    });
                                } else {
                                    web_sys::console::warn_1(&"⚠️ field_id is empty, cannot persist option".into());
                                }
                            };
                            
                            // Handle delete option - when user clicks delete button on an option
                            let handle_delete_option = {
                                let entity = entity_type_stored.get_value();
                                let field_id = field_id_stored.get_value();
                                move |value_to_delete: String| {
                                    // Remove from shared options immediately
                                    set_shared_options.update(|opts| {
                                        opts.retain(|o| o.value != value_to_delete);
                                    });
                                    
                                    // Persist deletion to backend
                                    if !field_id.is_empty() {
                                        let entity = entity.clone();
                                        let field_id = field_id.clone();
                                        let val = value_to_delete.clone();
                                        spawn_local(async move {
                                            match delete_field_option(&entity, &field_id, &val).await {
                                                Ok(_) => {
                                                    web_sys::console::log_1(&format!("✅ Deleted option '{}' from field {}", val, field_id).into());
                                                }
                                                Err(e) => {
                                                    web_sys::console::error_1(&format!("❌ Failed to delete option: {}", e).into());
                                                }
                                            }
                                        });
                                    }
                                }
                            };
                            view! {
                                // Stop propagation to prevent td click handler from interfering
                                <div class="w-full min-w-[140px]" on:click=|ev| ev.stop_propagation()>
                                    <SmartSelect
                                        options=select_options
                                        value=current
                                        on_change=handle_select_change
                                        allow_search=true
                                        allow_create=true
                                        on_create_value=Callback::new(handle_create_value)
                                        on_delete_option=Callback::new(handle_delete_option)
                                        create_label="+ Add New".to_string()
                                        placeholder="Search or type to add...".to_string()
                                    />
                                </div>
                            }.into_view()
                        }
                        "number" | "integer" | "decimal" | "money" => {
                            let current = local_value.get_value().as_f64().unwrap_or(0.0).to_string();
                            view! {
                                <div class="flex items-center gap-1 w-full">
                                    <input
                                        type="number"
                                        class="bg-transparent border-none outline-none text-white w-full p-0 h-full focus:ring-0"
                                        value=current
                                        autofocus=true
                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:mousedown=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:input=move |ev| {
                                            if let Ok(n) = event_target_value(&ev).parse::<f64>() {
                                                local_value.set_value(serde_json::json!(n));
                                            }
                                        }
                                        on:blur=handle_blur
                                        on:keydown=handle_keydown
                                    />
                                    <button class="ml-2 w-6 h-6 flex items-center justify-center rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-colors" on:click=handle_confirm>"✓"</button>
                                </div>
                            }.into_view()
                        }
                        "boolean" => {
                            let checked = local_value.get_value().as_bool().unwrap_or(false);
                            view! {
                                <div class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="rounded border-slate-600 bg-white/5 text-indigo-500 focus:ring-indigo-500 focus:ring-offset-0 cursor-pointer"
                                        checked=checked
                                        on:change=move |ev| {
                                            let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                            local_value.set_value(serde_json::json!(target.checked()));
                                        }
                                    />
                                    <button class="w-6 h-6 flex items-center justify-center rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-colors" on:click=handle_confirm>"✓"</button>
                                </div>
                            }.into_view()
                        }
                        "date" | "datetime" => {
                            // Date input - use native date picker
                            let current = local_value.get_value().as_str().unwrap_or("").to_string();
                            // Only take the date part for date input (YYYY-MM-DD)
                            let date_value = if current.len() >= 10 {
                                current[..10].to_string()
                            } else {
                                current.clone()
                            };
                            view! {
                                <div class="flex items-center gap-1 w-full">
                                    <input
                                        type="date"
                                        class="bg-transparent border-none outline-none text-white w-full p-0 h-full focus:ring-0 calendar-picker-indicator:filter-invert"
                                        value=date_value
                                        autofocus=true
                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:mousedown=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:input=move |ev| {
                                            local_value.set_value(serde_json::Value::String(event_target_value(&ev)));
                                        }
                                        on:blur=handle_blur
                                        on:keydown=handle_keydown
                                    />
                                    <button class="ml-2 w-6 h-6 flex items-center justify-center rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-colors" on:click=handle_confirm>"✓"</button>
                                </div>
                            }.into_view()
                        }
                        "link" => {
                            // Link field - use AsyncEntitySelect with lookup endpoint
                            let _current = local_value.get_value().as_str().unwrap_or("").to_string();
                            
                            // Get target entity from field_type definition
                            let _current = local_value.get_value().as_str().unwrap_or("").to_string();
                            
                            // Get target entity from stored value
                            let target_entity = target_entity_stored.get_value();
                            
                            // Setup Signal adapter for AsyncEntitySelect (requires RwSignal<Option<Uuid>>)
                            let initial_uuid = local_value.get_value().as_str()
                                .and_then(|s| uuid::Uuid::parse_str(s).ok());
                            let link_signal = create_rw_signal(initial_uuid);

                            // Sync back to table cell value when changed
                            create_effect(move |_| {
                                if let Some(uid) = link_signal.get() {
                                    // Verify it changed before saving to avoid loops/double save
                                    let current_str = local_value.get_value().as_str().unwrap_or("").to_string();
                                    if current_str != uid.to_string() {
                                        local_value.set_value(serde_json::Value::String(uid.to_string()));
                                        save_cell();
                                        set_editing_cell.set(None);
                                    }
                                }
                            });

                            view! {
                                <div class="w-full min-w-[200px]" on:click=|ev| ev.stop_propagation()>
                                    <AsyncEntitySelect
                                        entity_type=target_entity
                                        value=link_signal
                                        placeholder="Select...".to_string()
                                    />
                                </div>
                            }.into_view()
                        }
                        _ => {
                            // Default text input
                            let current = local_value.get_value().as_str().unwrap_or("").to_string();
                            view! {
                                <div class="flex items-center gap-1 w-full">
                                    <input
                                        type="text"
                                        class="bg-transparent border-none outline-none text-white w-full p-0 h-full focus:ring-0"
                                        value=current
                                        autofocus=true
                                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:mousedown=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                                        on:input=move |ev| {
                                            local_value.set_value(serde_json::Value::String(event_target_value(&ev)));
                                        }
                                        on:blur=handle_blur
                                        on:keydown=handle_keydown
                                    />
                                    <button class="ml-2 w-6 h-6 flex items-center justify-center rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-colors" on:click=handle_confirm>"✓"</button>
                                </div>
                            }.into_view()
                        }
                    }
                } else {
                    // Display mode
                    match ft.to_lowercase().as_str() {
                        "link" => {
                            let current = local_value.get_value().as_str().unwrap_or("").to_string();
                            let target = target_entity_stored.get_value();
                            view! { <AsyncEntityLabel entity_type=target id=current /> }.into_view()
                        },
                        _ => render_cell_display(&ft, &local_value.get_value())
                    }
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
            let status_color = match text.to_lowercase().as_str() {
                "new" | "open" | "active" => "bg-blue-500/10 text-blue-400 border-blue-500/20",
                "in_progress" | "pending" | "working" => "bg-amber-500/10 text-amber-400 border-amber-500/20",
                "won" | "completed" | "closed" | "success" => "bg-green-500/10 text-green-400 border-green-500/20",
                "lost" | "cancelled" | "failed" => "bg-red-500/10 text-red-400 border-red-500/20",
                "qualified" | "contacted" => "bg-purple-500/10 text-purple-400 border-purple-500/20",
                _ => "bg-slate-500/10 text-slate-400 border-slate-500/20",
            };
            view! {
                <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border {}", status_color)>
                    {text.to_string()}
                </span>
            }.into_view()
        }
        "boolean" => {
            let checked = value.as_bool().unwrap_or(false);
            view! {
                <span class=if checked { "text-green-400 font-bold" } else { "text-slate-600" }>
                    {if checked { "✓" } else { "—" }}
                </span>
            }.into_view()
        }
        "money" | "currency" => {
            let amount = value.as_f64().unwrap_or(0.0);
            view! {
                <span class="font-mono text-emerald-400">
                    {format!("${:.2}", amount)}
                </span>
            }.into_view()
        }
        "date" | "datetime" => {
            let date_str = value.as_str().unwrap_or("");
            // Format date nicely - shows only date part if present
            let display_date = if date_str.len() >= 10 {
                // Could format as "Dec 19, 2024" etc, but for now show YYYY-MM-DD
                date_str[..10].to_string()
            } else if date_str.is_empty() {
                "—".to_string()
            } else {
                date_str.to_string()
            };
            view! {
                <span class="text-slate-400 font-mono text-xs">{display_date}</span>
            }.into_view()
        }
        "email" => {
            let email = value.as_str().unwrap_or("");
            view! {
                <a class="text-indigo-400 hover:text-indigo-300 hover:underline" href=format!("mailto:{}", email)>
                    {email.to_string()}
                </a>
            }.into_view()
        }
        "phone" => {
            let phone = value.as_str().unwrap_or("");
            view! {
                <a class="text-indigo-400 hover:text-indigo-300 hover:underline" href=format!("tel:{}", phone)>
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
                <span class="truncate block max-w-[200px]" title=text.clone()>{text}</span>
            }.into_view()
        }
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
        <div class="w-full overflow-x-auto rounded-xl border border-white/10 bg-surface/30 backdrop-blur-md">
            <table class="w-full border-collapse text-left">
                <thead>
                    <tr>
                        {visible_columns.iter().map(|col| {
                            let label = field_map
                                .get(&col.field)
                                .map(|f| f.label.clone())
                                .unwrap_or_else(|| col.field.clone());
                            view! {
                                <th class="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500 border-b border-white/10 whitespace-nowrap">
                                    {label}
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody class="divide-y divide-white/5">
                    {data.iter().map(|row| {
                        view! {
                            <tr class="group transition-colors hover:bg-white/5">
                                {visible_columns.iter().map(|col| {
                                    let value = row.get(&col.field)
                                        .cloned()
                                        .unwrap_or(serde_json::Value::Null);
                                    let display = value.as_str().map(|s| s.to_string()).unwrap_or_else(|| value.to_string());
                                    view! {
                                        <td class="px-6 py-4 text-sm text-slate-300 whitespace-nowrap">
                                            {display}
                                        </td>
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
