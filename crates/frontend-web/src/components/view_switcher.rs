//! View Switcher Component - Toggle between Table/Kanban/Calendar/Map views
//! Reads available views from ViewDef API and allows switching
//! Persists selected view to localStorage per entity type

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use crate::api::{fetch_json, API_BASE, TENANT_ID};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewDefResponse {
    pub id: String,
    pub name: String,
    pub label: String,
    pub view_type: String,
    pub is_default: bool,
    pub settings: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewListResponse {
    pub views: Vec<ViewDefResponse>,
    pub total: i64,
}

/// Get saved view ID from localStorage for an entity type
fn get_saved_view(entity_type: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let key = format!("viewswitcher_{}", entity_type);
    storage.get_item(&key).ok()?
}

/// Save view ID to localStorage for an entity type
fn save_view(entity_type: &str, view_id: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let key = format!("viewswitcher_{}", entity_type);
            let _ = storage.set_item(&key, view_id);
        }
    }
}

/// Get default views for all view types
fn get_default_views() -> Vec<ViewDefResponse> {
    vec![
        ViewDefResponse {
            id: "default_table".to_string(),
            name: "default_table".to_string(),
            label: "Table".to_string(),
            view_type: "table".to_string(),
            is_default: true,
            settings: serde_json::json!({}),
        },
        ViewDefResponse {
            id: "default_kanban".to_string(),
            name: "default_kanban".to_string(),
            label: "Kanban".to_string(),
            view_type: "kanban".to_string(),
            is_default: false,
            settings: serde_json::json!({"group_field": "lifecycle_stage"}),
        },
        ViewDefResponse {
            id: "default_calendar".to_string(),
            name: "default_calendar".to_string(),
            label: "Calendar".to_string(),
            view_type: "calendar".to_string(),
            is_default: false,
            settings: serde_json::json!({"date_field": "created_at"}),
        },
        ViewDefResponse {
            id: "default_map".to_string(),
            name: "default_map".to_string(),
            label: "Map".to_string(),
            view_type: "map".to_string(),
            is_default: false,
            settings: serde_json::json!({}),
        },
    ]
}

#[component]
pub fn ViewSwitcher(
    entity_type: String,
    #[prop(into)] on_view_change: Callback<ViewDefResponse>,
) -> impl IntoView {
    let _entity_type_stored = store_value(entity_type.clone());
    
    // State
    let (views, set_views) = create_signal::<Vec<ViewDefResponse>>(Vec::new());
    let (active_view, set_active_view) = create_signal::<Option<String>>(None);
    let (loading, set_loading) = create_signal(true);
    
    // Fetch available views for this entity type
    let entity_for_effect = entity_type.clone();
    let entity_for_storage = entity_type.clone();
    let on_view_change_effect = on_view_change.clone();
    create_effect(move |_| {
        let et = entity_for_effect.clone();
        let et_storage = entity_for_storage.clone();
        let callback = on_view_change_effect.clone();
        
        spawn_local(async move {
            set_loading.set(true);
            
            let url = format!("{}/views?tenant_id={}&entity_type={}", API_BASE, TENANT_ID, et);
            match fetch_json::<ViewListResponse>(&url).await {
                Ok(response) => {
                    let mut view_list = response.views;
                    
                    // If no views returned, add default views for all types
                    if view_list.is_empty() {
                        view_list = get_default_views();
                    }
                    
                    // Try to restore saved view from localStorage
                    let saved_view_id = get_saved_view(&et_storage);
                    let selected_view = if let Some(ref saved_id) = saved_view_id {
                        view_list.iter().find(|v| &v.id == saved_id)
                    } else {
                        None
                    };
                    
                    // Use saved view, or default view, or first view
                    let view_to_use = selected_view
                        .or_else(|| view_list.iter().find(|v| v.is_default))
                        .or_else(|| view_list.first());
                    
                    if let Some(view) = view_to_use {
                        set_active_view.set(Some(view.id.clone()));
                        callback.call(view.clone());
                    }
                    
                    set_views.set(view_list);
                    set_loading.set(false);
                }
                Err(_) => {
                    // On error, show all default view types
                    let default_views = get_default_views();
                    set_active_view.set(Some("default_table".to_string()));
                    callback.call(default_views[0].clone());
                    set_views.set(default_views);
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Get view type icon
    let get_view_icon = |view_type: &str| -> &'static str {
        match view_type {
            "table" => "📋",
            "kanban" => "📊",
            "calendar" => "📅",
            "map" => "🗺️",
            "board" => "📌",
            "timeline" => "📈",
            "gallery" => "🖼️",
            _ => "📄",
        }
    };
    
    // Entity type for saving preference
    let entity_type_for_view = entity_type.clone();
    
    view! {
        <div class="view-switcher">
            {move || {
                let et_save = entity_type_for_view.clone();
                if loading.get() {
                    view! { <div class="view-switcher-loading"></div> }.into_view()
                } else {
                    view! {
                        <div class="view-tabs">
                            <For
                                each=move || views.get()
                                key=|v| v.id.clone()
                                children=move |view_def| {
                                    let view_id = view_def.id.clone();
                                    let view_id_click = view_def.id.clone();
                                    let view_id_save = view_def.id.clone();
                                    let view_type = view_def.view_type.clone();
                                    let view_label = view_def.label.clone();
                                    let view_def_click = view_def.clone();
                                    let is_active = move || active_view.get() == Some(view_id.clone());
                                    let icon = get_view_icon(&view_type);
                                    let et_for_save = et_save.clone();
                                    
                                    view! {
                                        <button
                                            class=move || format!("view-tab {}", if is_active() { "active" } else { "" })
                                            on:click=move |_| {
                                                set_active_view.set(Some(view_id_click.clone()));
                                                on_view_change.call(view_def_click.clone());
                                                // Persist selection to localStorage
                                                save_view(&et_for_save, &view_id_save);
                                            }
                                        >
                                            <span class="view-icon">{icon}</span>
                                            <span class="view-label">{view_label.clone()}</span>
                                        </button>
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

/// Simplified view type enum for rendering
#[derive(Clone, Debug, PartialEq)]
pub enum ViewType {
    Table,
    Kanban,
    Calendar,
    Map,
}

impl From<&str> for ViewType {
    fn from(s: &str) -> Self {
        match s {
            "kanban" => ViewType::Kanban,
            "calendar" => ViewType::Calendar,
            "map" => ViewType::Map,
            _ => ViewType::Table,
        }
    }
}
