//! Map View Component - Metadata-driven map view
//! Displays records on a map using Leaflet via JS interop

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::api::fetch_entity_list;

// Binding to global L (Leaflet)
// Binding to global L (Leaflet)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = "L", js_name = map)]
    fn leaflet_map(id: &str) -> MapObj;
    
    #[wasm_bindgen(js_namespace = "L")]
    fn tileLayer(url: &str, options: &JsValue) -> LayerObj;
    
    #[wasm_bindgen(js_namespace = "L", js_name = marker)]
    fn leaflet_marker(coords: &js_sys::Array) -> MarkerObj;
    
    #[wasm_bindgen(js_namespace = "L")]
    fn icon(options: &JsValue) -> IconObj;
}

#[wasm_bindgen]
extern "C" {
    type MapObj;
    #[wasm_bindgen(method)]
    fn setView(this: &MapObj, center: &js_sys::Array, zoom: u8) -> MapObj;
    #[wasm_bindgen(method)]
    fn addLayer(this: &MapObj, layer: &LayerObj) -> MapObj;
}

#[wasm_bindgen]
extern "C" {
    type LayerObj;
    #[wasm_bindgen(method)]
    fn addTo(this: &LayerObj, map: &MapObj) -> LayerObj;
}

#[wasm_bindgen]
extern "C" {
    type MarkerObj;
    #[wasm_bindgen(method)]
    fn addTo(this: &MarkerObj, map: &MapObj) -> MarkerObj;
    #[wasm_bindgen(method)]
    fn bindPopup(this: &MarkerObj, content: &str) -> MarkerObj;
    #[wasm_bindgen(method)]
    fn setIcon(this: &MarkerObj, icon: &IconObj) -> MarkerObj;
}

#[wasm_bindgen]
extern "C" {
    type IconObj;
}

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
    let config_stored = store_value(config.clone());
    let map_id = format!("map-{}", uuid::Uuid::new_v4());
    
    // State
    let (markers, set_markers) = create_signal::<Vec<MapMarker>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    // Fetch records
    let entity_for_effect = entity_type.clone();
    create_effect(move |_| {
        let et = entity_for_effect.clone();
        let cfg = config_stored.get_value();
        
        spawn_local(async move {
            let _ = set_loading.try_set(true);
            let _ = set_error.try_set(None);
            
            match fetch_entity_list(&et).await {
                Ok(response) => {
                    let records = response.data;
                    let map_markers: Vec<MapMarker> = records.into_iter()
                        .filter_map(|record| {
                            // Get lat/lng - try numeric first, then string parse
                            let lat = record.get(&cfg.lat_field)
                                .and_then(|v| v.as_f64())
                                .or_else(|| record.get(&cfg.lat_field).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()))?;
                                
                            let lng = record.get(&cfg.lng_field)
                                .and_then(|v| v.as_f64())
                                .or_else(|| record.get(&cfg.lng_field).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()))?;
                            
                            // Basic validation
                            if lat == 0.0 && lng == 0.0 { return None; }

                            let title = record.get(&cfg.popup_title_field)
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            let color = cfg.marker_color_field.as_ref()
                                .and_then(|f| record.get(f))
                                .and_then(|v| v.as_str())
                                .map(|s| get_marker_color(s))
                                .unwrap_or_else(|| "blue".to_string());
                            
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
                            let popup_html = format!("<div class='map-popup'><h4>{}</h4>{}</div>", title, popup_fields.join("<br>"));
                            
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
                    
                    let _ = set_markers.try_set(map_markers);
                    let _ = set_loading.try_set(false);
                }
                Err(e) => {
                    let _ = set_error.try_set(Some(e));
                    let _ = set_loading.try_set(false);
                }
            }
        });
    });
    
    // Initialize Map Result
    let map_id_clone = map_id.clone();
    
    // Effect to render map when markers change
    create_effect(move |_| {
        // Use try_get to avoid panic if owner is disposed
        let Some(current_markers) = markers.try_get() else { return };
        let Some(is_loading) = loading.try_get() else { return };
        if is_loading { return; }
        
        // We need a slight delay to ensure DOM is ready
        let m_id = map_id_clone.clone();
        let cfg = config_stored.get_value();
        
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                &Closure::once_into_js(move || {
                    init_map(&m_id, &current_markers, &cfg);
                }).into(),
                100, // 100ms delay
            );
        }
    });

    view! {
        <div class="map-view-wrapper bg-surface rounded-lg overflow-hidden" style="height: calc(100vh - 140px); width: 100%;">
            // Header with marker count
            <div class="map-header flex items-center justify-between px-4 py-2 border-b border-default">
                <div class="flex items-center gap-3">
                    <span class="text-primary font-semibold">"🗺️ Map View"</span>
                    <span class="ui-badge ui-badge-info">
                        {move || format!("{} properties", markers.get().len())}
                    </span>
                </div>
                // Legend
                <div class="map-legend flex items-center gap-4 text-xs">
                    <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-green-500"></span> "Active"</span>
                    <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-orange-500"></span> "Reserved"</span>
                    <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-red-500"></span> "Sold"</span>
                    <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-blue-500"></span> "Rented"</span>
                </div>
            </div>
            
            // Map container
            <div class="map-container" style="height: calc(100% - 40px); width: 100%; position: relative;">
                {move || {
                    if loading.get() {
                        view! { 
                            <div class="flex items-center justify-center h-full bg-surface">
                                <div class="text-center text-secondary">
                                    <div class="text-4xl mb-2 animate-pulse">"🗺️"</div>
                                    <div>"Loading map data..."</div>
                                </div>
                            </div>
                        }.into_view()
                    } else if let Some(err) = error.get() {
                        view! { 
                            <div class="flex items-center justify-center h-full bg-surface">
                                <div class="text-center">
                                    <div class="text-4xl mb-2">"❌"</div>
                                    <div class="text-error">{err}</div>
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                           <div id=map_id.clone() style="height: 100%; width: 100%; z-index: 1;"></div>
                        }.into_view()
                    }
                }}
            </div>
        </div>
    }
}

fn init_map(map_id: &str, markers: &[MapMarker], config: &MapConfig) {
    // Check if element exists
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    if document.get_element_by_id(map_id).is_none() {
        return;
    }

    // Default center (Dubai) or config
    let (lat, lng) = config.default_center.unwrap_or((25.2048, 55.2708));
    let zoom = config.default_zoom.unwrap_or(11);
    
    // Initialize map
    let center_arr = js_sys::Array::new();
    center_arr.push(&JsValue::from(lat));
    center_arr.push(&JsValue::from(lng));
    
    // We need to check if map is already initialized on this element, but for now we assume fresh mount or rely on Leaflet handling re-init gracefully (or we should destroy prev map).
    // In a robust implementation we'd store the map instance in a signal.
    
    // Create map
    let map_instance = leaflet_map(map_id).setView(&center_arr, zoom);
    
    // Add tile layer (OpenStreetMap)
    let tile_opts = js_sys::Object::new();
    js_sys::Reflect::set(&tile_opts, &JsValue::from("attribution"), &JsValue::from("&copy; OpenStreetMap contributors")).unwrap();
    
    tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", &tile_opts)
        .addTo(&map_instance);
        
    // Add markers
    for marker_data in markers {
        let pos = js_sys::Array::new();
        pos.push(&JsValue::from(marker_data.lat));
        pos.push(&JsValue::from(marker_data.lng));
        
        let m = leaflet_marker(&pos).addTo(&map_instance);
        m.bindPopup(&marker_data.popup_html);
        
        // Optional: Custom icons based on color could go here
    }
}

fn get_marker_color(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "draft" => "gray".to_string(),
        "active" => "green".to_string(),
        "reserved" | "under_offer" => "orange".to_string(),
        "sold" => "red".to_string(),
        "rented" => "blue".to_string(),
        _ => "blue".to_string(),
    }
}
