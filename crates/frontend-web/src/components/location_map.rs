//! Location Map - Basic Leaflet.js integration for location fields
//!
//! Features:
//! - Display marker on map
//! - Click to set location
//! - Draggable marker
//! - OpenStreetMap tiles

use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = L)]
    type Map;
    
    #[wasm_bindgen(js_namespace = L, js_name = map)]
    fn create_map(id: &str) -> Map;
    
    #[wasm_bindgen(method, js_name = setView)]
    fn set_view(this: &Map, latlng: &JsValue, zoom: u32);
    
    #[wasm_bindgen(js_namespace = L)]
    type TileLayer;
    
    #[wasm_bindgen(js_namespace = L, js_name = tileLayer)]
    fn tile_layer(url: &str, options: &JsValue) -> TileLayer;
    
    #[wasm_bindgen(method, js_name = addTo)]
    fn add_to_map(this: &TileLayer, map: &Map);
    
    #[wasm_bindgen(js_namespace = L)]
    type Marker;
    
    #[wasm_bindgen(js_namespace = L, js_name = marker)]
    fn create_marker(latlng: &JsValue, options: &JsValue) -> Marker;
    
    #[wasm_bindgen(method, js_name = addTo)]
    fn add_marker_to_map(this: &Marker, map: &Map);
}

#[component]
pub fn LocationMap(
    /// Latitude
    #[prop(into)] lat: Signal<f64>,
    /// Longitude
    #[prop(into)] lng: Signal<f64>,
    /// Callback when location changes
    #[prop(optional)] on_change: Option<Callback<(f64, f64)>>,
    /// Map height
    #[prop(default = "400px".to_string())] height: String,
    /// Allow editing
    #[prop(default = true)] editable: bool,
) -> impl IntoView {
    let map_id = format!("map-{}", uuid::Uuid::new_v4());
    let map_id_clone = map_id.clone();
    
    // Note: In a real implementation, you would:
    // 1. Add Leaflet CSS/JS to index.html
    // 2. Initialize the map in create_effect
    // 3. Add event listeners for clicks and marker dragging
    
    // Placeholder implementation
    create_effect(move |_| {
        let _lat_val = lat.get();
        let _lng_val = lng.get();
        
        // TODO: Initialize Leaflet map
        // This requires adding Leaflet to index.html first
        // Map would be initialized here with lat/lng
    });

    view! {
        <div class="location-map">
            <div
                id=map_id_clone
                class="map-container border border-gray-300 dark:border-gray-700 rounded-lg"
                style:height=height
            >
                // Placeholder content
                <div class="flex items-center justify-center h-full bg-gray-100 dark:bg-gray-800 text-gray-500">
                    <div class="text-center">
                        <div class="text-4xl mb-2">"🗺️"</div>
                        <div>"Map Component"</div>
                        <div class="text-sm mt-2">
                            "Lat: " {move || lat.get()} " | Lng: " {move || lng.get()}
                        </div>
                        <div class="text-xs text-gray-400 mt-2">
                            "Add Leaflet.js to index.html to enable interactive map"
                        </div>
                    </div>
                </div>
            </div>
            
            {if editable {
                view! {
                    <div class="mt-2 flex gap-2">
                        <input
                            type="number"
                            step="0.000001"
                            placeholder="Latitude"
                            value=move || lat.get()
                            class="form-input flex-1"
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                    if let Some(cb) = on_change {
                                        cb.call((val, lng.get()));
                                    }
                                }
                            }
                        />
                        <input
                            type="number"
                            step="0.000001"
                            placeholder="Longitude"
                            value=move || lng.get()
                            class="form-input flex-1"
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                    if let Some(cb) = on_change {
                                        cb.call((lat.get(), val));
                                    }
                                }
                            }
                        />
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}
