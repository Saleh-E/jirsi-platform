//! Map View Component - Metadata-driven map view
//! Displays records on a map using latitude/longitude fields

use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::{fetch_entity_list, API_BASE, TENANT_ID};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapConfig {
    pub lat_field: String,
    pub lng_field: String,
    pub popup_title_field: String,
    pub popup_fields: Vec<String>,
    pub marker_color_field: Option<String>,
    pub default_center: Option<(f64, f64)>,
    pub default_zoom: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct MapMarker {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    pub title: String,
    pub color: String,
    pub popup_html: String,
    pub record: serde_json::Value,
}

#[component]
pub fn MapView(
    entity_type: String,
    config: MapConfig,
) -> impl IntoView {
    let config_stored = store_value(config);
    
    // State
    let (markers, set_markers) = create_signal::<Vec<MapMarker>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (selected_marker, set_selected_marker) = create_signal::<Option<String>>(None);
    
    // Fetch records
    let entity_for_effect = entity_type.clone();
    create_effect(move |_| {
        let et = entity_for_effect.clone();
        let cfg = config_stored.get_value();
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            match fetch_entity_list(&et).await {
                Ok(response) => {
                    let records = response.data;
                    let map_markers: Vec<MapMarker> = records.into_iter()
                        .filter_map(|record| {
                            // Get lat/lng
                            let lat = record.get(&cfg.lat_field)
                                .and_then(|v| v.as_f64())?;
                            let lng = record.get(&cfg.lng_field)
                                .and_then(|v| v.as_f64())?;
                            
                            let title = record.get(&cfg.popup_title_field)
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            let color = cfg.marker_color_field.as_ref()
                                .and_then(|f| record.get(f))
                                .and_then(|v| v.as_str())
                                .map(|s| get_marker_color(s))
                                .unwrap_or_else(|| "#3b82f6".to_string());
                            
                            // Build popup HTML
                            let popup_fields: Vec<String> = cfg.popup_fields.iter()
                                .filter_map(|f| {
                                    record.get(f).map(|v| {
                                        let val = match v {
                                            serde_json::Value::String(s) => s.clone(),
                                            serde_json::Value::Number(n) => n.to_string(),
                                            serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
                                            _ => "-".to_string(),
                                        };
                                        format!("<b>{}:</b> {}", f, val)
                                    })
                                })
                                .collect();
                            let popup_html = format!("<h4>{}</h4>{}", title, popup_fields.join("<br>"));
                            
                            let id = record.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            
                            Some(MapMarker {
                                id,
                                lat,
                                lng,
                                title,
                                color,
                                popup_html,
                                record,
                            })
                        })
                        .collect();
                    
                    set_markers.set(map_markers);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Default center (Dubai)
    let default_center = config_stored.get_value().default_center.unwrap_or((25.2048, 55.2708));
    let default_zoom = config_stored.get_value().default_zoom.unwrap_or(11);
    
    view! {
        <div class="map-container">
            {move || {
                if loading.get() {
                    view! { <div class="map-loading">"Loading map..."</div> }.into_view()
                } else if let Some(err) = error.get() {
                    view! { <div class="map-error">{err}</div> }.into_view()
                } else {
                    let marker_count = markers.get().len();
                    view! {
                        <div class="map-wrapper">
                            <div class="map-header">
                                <span class="marker-count">{format!("{} properties on map", marker_count)}</span>
                            </div>
                            // Map placeholder - in production would use Leaflet/MapLibre
                            <div class="map-placeholder" id="map-canvas" style=format!(
                                "background: linear-gradient(135deg, #1a365d 0%, #2d3748 100%); display: flex; align-items: center; justify-content: center; color: white; flex-direction: column;"
                            )>
                                <div style="font-size: 3rem; margin-bottom: 1rem;">"🗺️"</div>
                                <p style="text-align: center; max-width: 300px;">
                                    "Map component ready."<br/>
                                    <small style="opacity: 0.7;">{format!("Center: {:.4}, {:.4} | Zoom: {}", default_center.0, default_center.1, default_zoom)}</small>
                                </p>
                            </div>
                            // Marker list sidebar
                            <div class="map-sidebar">
                                <h4 class="sidebar-title">"Properties"</h4>
                                <div class="marker-list">
                                    <For
                                        each=move || markers.get()
                                        key=|m| m.id.clone()
                                        children=move |marker| {
                                            let marker_id = marker.id.clone();
                                            let is_selected = move || selected_marker.get() == Some(marker_id.clone());
                                            let bg_color = marker.color.clone();
                                            
                                            view! {
                                                <div 
                                                    class=move || format!("marker-item {}", if is_selected() { "selected" } else { "" })
                                                    on:click={
                                                        let id = marker.id.clone();
                                                        move |_| {
                                                            set_selected_marker.set(Some(id.clone()));
                                                        }
                                                    }
                                                >
                                                    <span class="marker-dot" style=format!("background-color: {}", bg_color)></span>
                                                    <span class="marker-title">{marker.title.clone()}</span>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

fn get_marker_color(status: &str) -> String {
    match status {
        "draft" => "#6b7280".to_string(),
        "active" => "#22c55e".to_string(),
        "reserved" | "under_offer" => "#f59e0b".to_string(),
        "sold" => "#10b981".to_string(),
        "rented" => "#06b6d4".to_string(),
        "withdrawn" | "expired" => "#ef4444".to_string(),
        _ => "#3b82f6".to_string(),
    }
}
