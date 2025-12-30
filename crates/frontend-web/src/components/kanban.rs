//! Kanban View Component - Metadata-driven board view
//! Groups records by a select field and displays them as draggable cards

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

#[component]
pub fn KanbanView(
    entity_type: String,
    config: KanbanConfig,
    #[prop(optional)] field_options: Vec<(String, String)>,
    /// List of fields that can be used for grouping (value, label)
    #[prop(optional)] groupable_fields: Vec<(String, String)>,
    /// Callback when user changes the grouping field
    #[prop(optional, into)] on_group_change: Option<Callback<String>>,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type.clone());
    let config_stored = store_value(config.clone());
    let options_stored = store_value(field_options);
    let groupable_stored = store_value(groupable_fields);
    let on_group_change_stored = store_value(on_group_change);
    
    // Current grouping field (reactive to allow changes)
    let (current_group_field, set_current_group_field) = create_signal(config.group_by_field.clone());
    
    // State for columns and records
    let (columns, set_columns) = create_signal::<Vec<KanbanColumn>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (dragging_id, set_dragging_id) = create_signal::<Option<String>>(None);
    let (refresh_trigger, _set_refresh) = create_signal(0);
    
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
    
    view! {
        <div class="flex flex-col h-full">
            {move || {
                if loading.get() {
                    view! { <div class="flex items-center justify-center p-8 text-slate-400">"Loading..."</div> }.into_view()
                } else if let Some(err) = error.get() {
                    view! { <div class="p-4 text-red-400 bg-red-500/10 rounded-lg">{err}</div> }.into_view()
                } else {
                    let cfg = config_stored.get_value();
                    let groupable = groupable_stored.get_value();
                    let show_selector = !groupable.is_empty();
                    
                    view! {
                        // Kanban Header with Field Selector
                        <Show when=move || show_selector>
                            <div class="flex items-center justify-between mb-4 px-2">
                                <div class="flex items-center gap-2">
                                    <span class="text-slate-400 text-sm">"Group by:"</span>
                                    <select
                                        class="bg-surface border border-white/10 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                        on:change=move |ev| {
                                            let new_field = event_target_value(&ev);
                                            set_current_group_field.set(new_field.clone());
                                            if let Some(cb) = on_group_change_stored.get_value() {
                                                cb.call(new_field);
                                            }
                                        }
                                    >
                                        {groupable_stored.get_value().into_iter().map(|(val, label)| {
                                            let is_selected = val == current_group_field.get();
                                            view! {
                                                <option value=val.clone() selected=is_selected>{label}</option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </div>
                                <span class="text-slate-500 text-xs">
                                    {move || format!("{} cards", columns.get().iter().map(|c| c.records.len()).sum::<usize>())}
                                </span>
                            </div>
                        </Show>
                        
                        <div class="flex gap-4 overflow-x-auto pb-4 custom-scrollbar">
                            <For
                                each=move || columns.get()
                                key=|col| col.value.clone()
                                children=move |column| {
                                    let column_value_drop = column.value.clone();
                                    let column_color = column.color.clone();
                                    let record_count = column.records.len();
                                    let column_label = column.label.clone();
                                    let records = column.records.clone();
                                    
                                    view! {
                                        <div 
                                            class="flex-shrink-0 w-72 bg-surface/50 backdrop-blur-lg rounded-xl border border-white/10 overflow-hidden"
                                            style=format!("--column-color: {}", column_color)
                                            on:dragover=move |ev| {
                                                ev.prevent_default();
                                            }
                                            on:drop=move |ev| {
                                                ev.prevent_default();
                                                if let Some(record_id) = dragging_id.get() {
                                                    let et = entity_type_stored.get_value();
                                                    let cfg = config_stored.get_value();
                                                    let new_status = column_value_drop.clone();
                                                    
                                                    spawn_local(async move {
                                                        let mut data = serde_json::Map::new();
                                                        data.insert(cfg.group_by_field.clone(), serde_json::json!(new_status));
                                                        
                                                        let url = format!(
                                                            "{}/records/{}/{}",
                                                            API_BASE, et, record_id
                                                        );
                                                        
                                                        match put_json::<serde_json::Value>(
                                                            &url,
                                                            &serde_json::Value::Object(data)
                                                        ).await {
                                                            Ok(_) => {
                                                                logging::log!("Kanban: Updated status");
                                                            }
                                                            Err(e) => {
                                                                logging::log!("Failed to update: {}", e);
                                                            }
                                                        }
                                                    });
                                                    
                                                    set_dragging_id.set(None);
                                                }
                                            }
                                        >
                                            <div class="p-3 border-b border-white/10 flex justify-between items-center bg-white/5">
                                                <div class="flex items-center gap-2">
                                                    <span 
                                                        class="w-3 h-3 rounded-full" 
                                                        style=format!("background-color: {}", column_color)
                                                    ></span>
                                                    <span class="font-medium text-white">{column_label}</span>
                                                </div>
                                                <span class="bg-white/10 text-slate-300 text-xs px-2 py-0.5 rounded-full">
                                                    {record_count}
                                                </span>
                                            </div>
                                            <div class="p-2 space-y-2 max-h-[60vh] overflow-y-auto custom-scrollbar">
                                                {records.into_iter().map(|record| {
                                                    let record_id = record.get("id")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let drag_id = record_id.clone();
                                                    let title = record.get(&cfg.card_title_field)
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("Untitled")
                                                        .to_string();
                                                    
                                                    view! {
                                                        <div 
                                                            class="bg-surface rounded-lg p-3 cursor-move hover:bg-white/10 transition-colors border border-white/5 shadow-sm"
                                                            draggable="true"
                                                            on:dragstart=move |_| {
                                                                set_dragging_id.set(Some(drag_id.clone()));
                                                            }
                                                        >
                                                            <div class="font-medium text-white text-sm">{title}</div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    }.into_view()
                }
            }}
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
