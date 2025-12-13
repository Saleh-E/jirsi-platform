//! View Switcher Component - Toggle between Table/Kanban/Calendar/Map views
//! Reads available views from ViewDef API and allows switching

use leptos::*;
use serde::{Deserialize, Serialize};
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

#[component]
pub fn ViewSwitcher(
    entity_type: String,
    #[prop(into)] on_view_change: Callback<ViewDefResponse>,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type.clone());
    
    // State
    let (views, set_views) = create_signal::<Vec<ViewDefResponse>>(Vec::new());
    let (active_view, set_active_view) = create_signal::<Option<String>>(None);
    let (loading, set_loading) = create_signal(true);
    
    // Fetch available views for this entity type
    let entity_for_effect = entity_type.clone();
    let on_view_change_effect = on_view_change.clone();
    create_effect(move |_| {
        let et = entity_for_effect.clone();
        let callback = on_view_change_effect.clone();
        
        spawn_local(async move {
            set_loading.set(true);
            
            let url = format!("{}/views?tenant_id={}&entity_type={}", API_BASE, TENANT_ID, et);
            match fetch_json::<ViewListResponse>(&url).await {
                Ok(response) => {
                    let mut view_list = response.views;
                    
                    // If no views returned, add a default table view
                    if view_list.is_empty() {
                        view_list.push(ViewDefResponse {
                            id: "default_table".to_string(),
                            name: "default_table".to_string(),
                            label: "Table".to_string(),
                            view_type: "table".to_string(),
                            is_default: true,
                            settings: serde_json::json!({}),
                        });
                    }
                    
                    // Set default view as active and trigger callback
                    if let Some(default_view) = view_list.iter().find(|v| v.is_default) {
                        set_active_view.set(Some(default_view.id.clone()));
                        callback.call(default_view.clone());
                    } else if let Some(first) = view_list.first() {
                        set_active_view.set(Some(first.id.clone()));
                        callback.call(first.clone());
                    }
                    
                    set_views.set(view_list);
                    set_loading.set(false);
                }
                Err(_) => {
                    // On error, still show a default table view
                    let default_views = vec![ViewDefResponse {
                        id: "default_table".to_string(),
                        name: "default_table".to_string(),
                        label: "Table".to_string(),
                        view_type: "table".to_string(),
                        is_default: true,
                        settings: serde_json::json!({}),
                    }];
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
    
    view! {
        <div class="view-switcher">
            {move || {
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
                                    let view_type = view_def.view_type.clone();
                                    let view_label = view_def.label.clone();
                                    let view_def_click = view_def.clone();
                                    let is_active = move || active_view.get() == Some(view_id.clone());
                                    let icon = get_view_icon(&view_type);
                                    
                                    view! {
                                        <button
                                            class=move || format!("view-tab {}", if is_active() { "active" } else { "" })
                                            on:click=move |_| {
                                                set_active_view.set(Some(view_id_click.clone()));
                                                on_view_change.call(view_def_click.clone());
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
