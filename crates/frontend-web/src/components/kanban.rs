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
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type.clone());
    let config_stored = store_value(config);
    let options_stored = store_value(field_options);
    
    // State for columns and records
    let (columns, set_columns) = create_signal::<Vec<KanbanColumn>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (dragging_id, set_dragging_id) = create_signal::<Option<String>>(None);
    let (refresh_trigger, set_refresh) = create_signal(0);
    
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
        <div class="kanban-container">
            {move || {
                if loading.get() {
                    view! { <div class="kanban-loading">"Loading..."</div> }.into_view()
                } else if let Some(err) = error.get() {
                    view! { <div class="kanban-error">{err}</div> }.into_view()
                } else {
                    let cfg = config_stored.get_value();
                    view! {
                        <div class="kanban-board">
                            <For
                                each=move || columns.get()
                                key=|col| col.value.clone()
                                children=move |column| {
                                    let column_value_drop = column.value.clone();
                                    let column_color = column.color.clone();
                                    let record_count = column.records.len();
                                    let column_label = column.label.clone();
                                    
                                    view! {
                                        <div 
                                            class="kanban-column"
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
                                                        
                                                        // Correct URL: /records/{entity_code}/{record_id}
                                                        let url = format!(
                                                            "{}/records/{}/{}",
                                                            API_BASE, et, record_id
                                                        );
                                                        
                                                        match put_json::<serde_json::Value>(
                                                            &url,
                                                            &serde_json::Value::Object(data)
                                                        ).await {
                                                            Ok(_) => {
                                                                set_refresh.update(|n| *n += 1);
                                                            }
                                                            Err(e) => {
                                                                logging::log!("Failed to update Kanban status: {}", e);
                                                            }
                                                        }
                                                    });
                                                    
                                                    set_dragging_id.set(None);
                                                }
                                            }
                                        >
                                            <div class="kanban-column-header">
                                                <span class="column-title">{column_label}</span>
                                                <span class="column-count">{record_count}</span>
                                            </div>
                                            <div class="kanban-column-body">
                                                <For
                                                    each=move || column.records.clone()
                                                    key=|record| record.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()
                                                    children={
                                                        let cfg = cfg.clone();
                                                        move |record| {
                                                            let record_id = record.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
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
                                                            
                                                            view! {
                                                                <div 
                                                                    class="kanban-card"
                                                                    draggable="true"
                                                                    on:dragstart=move |_| {
                                                                        set_dragging_id.set(Some(drag_id.clone()));
                                                                    }
                                                                >
                                                                    <div class="card-title">{title}</div>
                                                                    {subtitle.map(|s| view! { <div class="card-subtitle">{s}</div> })}
                                                                    <div class="card-fields">
                                                                        {card_fields.into_iter()
                                                                            .map(|(k, v)| view! {
                                                                                <div class="card-field">
                                                                                    <span class="field-label">{k}</span>
                                                                                    <span class="field-value">{v}</span>
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
