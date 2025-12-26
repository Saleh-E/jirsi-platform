//! Kanban View Component - Metadata-driven board view
//! Groups records by a select field and displays them as draggable cards
//!
//! Features:
//! - Optimistic UI updates with rollback on error
//! - CQRS command dispatch for stage changes
//! - Drag-and-drop with visual feedback
//! - Real-time updates via WebSocket

use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::api::{fetch_entity_list, put_json, API_BASE};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KanbanConfig {
    pub group_by_field: String,
    pub card_title_field: String,
    pub card_subtitle_field: Option<String>,
    pub card_fields: Vec<String>,
    pub column_order: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct KanbanColumn {
    pub value: String,
    pub label: String,
    pub color: String,
    pub records: Vec<serde_json::Value>,
}

/// CQRS Command for stage updates (sent to backend)
#[derive(Clone, Debug, Serialize)]
pub struct UpdateStageCommand {
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

#[component]
pub fn KanbanView(
    entity_type: String,
    config: KanbanConfig,
    #[prop(optional)] field_options: Vec<(String, String)>,
    #[prop(optional)] on_card_click: Option<Callback<String>>,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type.clone());
    let config_stored = store_value(config);
    let options_stored = store_value(field_options);
    
    // State for columns and records
    let (columns, set_columns) = create_signal::<Vec<KanbanColumn>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (dragging_id, set_dragging_id) = create_signal::<Option<String>>(None);
    let (dragging_from_column, set_dragging_from_column) = create_signal::<Option<String>>(None);
    let (drop_target_column, set_drop_target_column) = create_signal::<Option<String>>(None);
    let (refresh_trigger, set_refresh) = create_signal(0);
    
    // Track cards being updated (show loading indicator)
    let (updating_cards, set_updating_cards) = create_signal::<Vec<String>>(Vec::new());
    
    // Fetch records and group by field
    let entity_for_effect = entity_type.clone();
    create_effect(move |_| {
        let _ = refresh_trigger.get(); // React to refresh
        let et = entity_for_effect.clone();
        let cfg = config_stored.get_value();
        let opts = options_stored.get_value();
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            match fetch_entity_list(&et).await {
                Ok(response) => {
                    let records: Vec<serde_json::Value> = response.data;
                    // Group records by the group_by_field
                    let mut grouped: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
                    
                    for record in records {
                        let group_value = record
                            .get(&cfg.group_by_field)
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        grouped.entry(group_value).or_default().push(record);
                    }
                    
                    // Create columns in order
                    let column_order: Vec<String> = cfg.column_order.clone().unwrap_or_else(|| {
                        opts.iter().map(|(v, _)| v.clone()).collect()
                    });
                    
                    let mut cols: Vec<KanbanColumn> = column_order
                        .iter()
                        .map(|val| {
                            let label = opts.iter()
                                .find(|(v, _)| v == val)
                                .map(|(_, l)| l.clone())
                                .unwrap_or_else(|| val.clone());
                            
                            KanbanColumn {
                                value: val.clone(),
                                label,
                                color: get_status_color(val),
                                records: grouped.remove(val).unwrap_or_default(),
                            }
                        })
                        .collect();
                    
                    // Add any remaining columns not in order
                    for (val, recs) in grouped {
                        cols.push(KanbanColumn {
                            value: val.clone(),
                            label: val.clone(),
                            color: get_status_color(&val),
                            records: recs,
                        });
                    }
                    
                    set_columns.set(cols);
                    set_loading.set(false);
                }
                Err(err_msg) => {
                    set_error.set(Some(err_msg));
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Optimistic Move Function
    let move_card_optimistic = move |record_id: String, from_col: String, to_col: String| {
        if from_col == to_col {
            return; // No change
        }
        
        // Optimistically update UI immediately
        set_columns.update(|cols| {
            let mut record_to_move: Option<serde_json::Value> = None;
            
            // Remove from source column
            for col in cols.iter_mut() {
                if col.value == from_col {
                    if let Some(idx) = col.records.iter().position(|r| {
                        r.get("id").and_then(|v| v.as_str()) == Some(&record_id)
                    }) {
                        record_to_move = Some(col.records.remove(idx));
                        break;
                    }
                }
            }
            
            // Add to target column
            if let Some(record) = record_to_move {
                for col in cols.iter_mut() {
                    if col.value == to_col {
                        col.records.push(record);
                        break;
                    }
                }
            }
        });
        
        // Mark card as updating
        set_updating_cards.update(|cards| cards.push(record_id.clone()));
        
        // Send request to backend
        let et = entity_type_stored.get_value();
        let cfg = config_stored.get_value();
        let record_id_for_api = record_id.clone();
        let from_col_rollback = from_col.clone();
        let to_col_rollback = to_col.clone();
        
        spawn_local(async move {
            let mut data = serde_json::Map::new();
            data.insert(cfg.group_by_field.clone(), serde_json::json!(to_col.clone()));
            
            // URL: /records/{entity_code}/{record_id}
            let url = format!(
                "{}/records/{}/{}",
                API_BASE, et, record_id_for_api
            );
            
            let result = put_json::<serde_json::Value>(
                &url,
                &serde_json::Value::Object(data)
            ).await;
            
            // Remove from updating cards
            set_updating_cards.update(|cards| {
                cards.retain(|id| id != &record_id_for_api);
            });
            
            match result {
                Ok(_) => {
                    // Success - UI already updated
                    logging::log!("Kanban: Card {} moved to {}", record_id_for_api, to_col);
                }
                Err(e) => {
                    // Rollback on error
                    logging::log!("Kanban: Failed to move card, rolling back: {}", e);
                    set_columns.update(|cols| {
                        let mut record_to_move: Option<serde_json::Value> = None;
                        
                        // Move back from target to source
                        for col in cols.iter_mut() {
                            if col.value == to_col_rollback {
                                if let Some(idx) = col.records.iter().position(|r| {
                                    r.get("id").and_then(|v| v.as_str()) == Some(&record_id_for_api)
                                }) {
                                    record_to_move = Some(col.records.remove(idx));
                                    break;
                                }
                            }
                        }
                        
                        if let Some(record) = record_to_move {
                            for col in cols.iter_mut() {
                                if col.value == from_col_rollback {
                                    col.records.push(record);
                                    break;
                                }
                            }
                        }
                    });
                    
                    set_error.set(Some(format!("Failed to update status: {}", e)));
                    
                    // Clear error after 3 seconds
                    set_timeout(move || {
                        set_error.set(None);
                    }, std::time::Duration::from_secs(3));
                }
            }
        });
    };
    
    view! {
        <div class="kanban-container relative">
            // Loading overlay
            <Show when=move || loading.get()>
                <div class="absolute inset-0 bg-gray-900/50 flex items-center justify-center z-10 rounded-lg">
                    <div class="flex items-center gap-3 text-white">
                        <i class="fa-solid fa-spinner fa-spin text-2xl"></i>
                        <span>"Loading board..."</span>
                    </div>
                </div>
            </Show>
            
            // Error toast
            <Show when=move || error.get().is_some()>
                <div class="absolute top-4 right-4 bg-danger-500 text-white px-4 py-2 rounded-lg shadow-lg z-20 animate-slide-in-right">
                    <div class="flex items-center gap-2">
                        <i class="fa-solid fa-exclamation-triangle"></i>
                        <span>{move || error.get().unwrap_or_default()}</span>
                    </div>
                </div>
            </Show>
            
            <div class="kanban-board flex gap-4 overflow-x-auto pb-4 min-h-[500px]">
                <For
                    each=move || columns.get()
                    key=|col| col.value.clone()
                    children=move |column| {
                        let column_value = column.value.clone();
                        let column_value_drop = column.value.clone();
                        let column_value_for_css = column.value.clone();
                        let column_color = column.color.clone();
                        let record_count = column.records.len();
                        let column_label = column.label.clone();
                        let is_drop_target = move || drop_target_column.get() == Some(column_value_for_css.clone());
                        
                        view! {
                            <div 
                                class="kanban-column flex-shrink-0 w-72 bg-gray-800 rounded-lg transition-all duration-200"
                                class:ring-2=is_drop_target
                                class:ring-brand-500=is_drop_target
                                class:scale-105=is_drop_target
                                style=format!("--column-color: {}", column_color)
                                on:dragover=move |ev| {
                                    ev.prevent_default();
                                    set_drop_target_column.set(Some(column_value.clone()));
                                }
                                on:dragleave=move |_| {
                                    set_drop_target_column.set(None);
                                }
                                on:drop=move |ev| {
                                    ev.prevent_default();
                                    set_drop_target_column.set(None);
                                    
                                    if let (Some(record_id), Some(from_col)) = (dragging_id.get(), dragging_from_column.get()) {
                                        move_card_optimistic(record_id, from_col, column_value_drop.clone());
                                    }
                                    
                                    set_dragging_id.set(None);
                                    set_dragging_from_column.set(None);
                                }
                            >
                                <div class="kanban-column-header p-3 border-b border-gray-700 flex items-center justify-between">
                                    <div class="flex items-center gap-2">
                                        <span 
                                            class="w-3 h-3 rounded-full" 
                                            style=format!("background-color: {}", column_color)
                                        ></span>
                                        <span class="font-medium text-white">{column_label}</span>
                                    </div>
                                    <span class="bg-gray-700 text-gray-300 text-xs px-2 py-1 rounded-full">
                                        {record_count}
                                    </span>
                                </div>
                                
                                <div class="kanban-column-body p-2 space-y-2 max-h-[calc(100vh-300px)] overflow-y-auto">
                                    <For
                                        each=move || column.records.clone()
                                        key=|record| record.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()
                                        children={
                                            let cfg = config_stored.get_value();
                                            let column_for_drag = column.value.clone();
                                            move |record| {
                                                let record_id = record.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let is_updating = move || updating_cards.get().contains(&record_id);
                                                let title = record.get(&cfg.card_title_field)
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("Untitled")
                                                    .to_string();
                                                let subtitle = cfg.card_subtitle_field.as_ref()
                                                    .and_then(|f| record.get(f))
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                                
                                                let card_fields: Vec<(String, String)> = cfg.card_fields.iter()
                                                    .filter_map(|f| {
                                                        record.get(f).map(|v| {
                                                            let val = match v {
                                                                serde_json::Value::String(s) => s.clone(),
                                                                serde_json::Value::Number(n) => n.to_string(),
                                                                serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
                                                                serde_json::Value::Null => "-".to_string(),
                                                                _ => v.to_string(),
                                                            };
                                                            (f.clone(), val)
                                                        })
                                                    })
                                                    .collect();
                                                
                                                let drag_id = record_id.clone();
                                                let drag_from_col = column_for_drag.clone();
                                                let click_id = record_id.clone();
                                                
                                                view! {
                                                    <div 
                                                        class="kanban-card bg-gray-700 rounded-lg p-3 cursor-move hover:bg-gray-650 transition-all duration-150 hover:shadow-lg relative"
                                                        class:opacity-50=is_updating
                                                        class:pointer-events-none=is_updating
                                                        draggable="true"
                                                        on:dragstart=move |_| {
                                                            set_dragging_id.set(Some(drag_id.clone()));
                                                            set_dragging_from_column.set(Some(drag_from_col.clone()));
                                                        }
                                                        on:dragend=move |_| {
                                                            set_dragging_id.set(None);
                                                            set_dragging_from_column.set(None);
                                                            set_drop_target_column.set(None);
                                                        }
                                                        on:click=move |_| {
                                                            if let Some(cb) = on_card_click {
                                                                cb.call(click_id.clone());
                                                            }
                                                        }
                                                    >
                                                        // Loading overlay for updating cards
                                                        <Show when=is_updating>
                                                            <div class="absolute inset-0 bg-gray-900/50 rounded-lg flex items-center justify-center">
                                                                <i class="fa-solid fa-spinner fa-spin text-brand-400"></i>
                                                            </div>
                                                        </Show>
                                                        
                                                        <div class="card-title font-medium text-white mb-1">{title}</div>
                                                        {subtitle.map(|s| view! { 
                                                            <div class="card-subtitle text-sm text-gray-400 mb-2">{s}</div> 
                                                        })}
                                                        <div class="card-fields text-xs space-y-1">
                                                            {card_fields.into_iter()
                                                                .take(3)
                                                                .map(|(k, v)| view! {
                                                                    <div class="card-field flex justify-between">
                                                                        <span class="text-gray-500">{k}</span>
                                                                        <span class="text-gray-300">{v}</span>
                                                                    </div>
                                                                })
                                                                .collect_view()
                                                            }
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        }
                                    />
                                    
                                    // Drop indicator when empty or hovering
                                    <Show when=move || column.records.is_empty() || is_drop_target()>
                                        <div class="kanban-drop-zone border-2 border-dashed border-gray-600 rounded-lg p-4 text-center text-gray-500 text-sm">
                                            <i class="fa-solid fa-plus mr-2"></i>
                                            "Drop here"
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

fn get_status_color(status: &str) -> String {
    match status {
        "draft" => "#6b7280".to_string(),
        "active" | "scheduled" | "submitted" => "#22c55e".to_string(),
        "pending" | "pending_signature" => "#f59e0b".to_string(),
        "reserved" | "confirmed" | "under_review" => "#3b82f6".to_string(),
        "under_offer" | "countered" => "#8b5cf6".to_string(),
        "sold" | "completed" | "accepted" => "#10b981".to_string(),
        "rented" => "#06b6d4".to_string(),
        "expired" | "withdrawn" | "rejected" => "#ef4444".to_string(),
        "cancelled" | "no_show" => "#dc2626".to_string(),
        "paused" => "#9ca3af".to_string(),
        _ => "#6b7280".to_string(),
    }
}

