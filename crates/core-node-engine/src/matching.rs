//! Smart Match Handler - AI-powered Lead ↔ Property matching
//!
//! Matches leads/buyers with properties based on:
//! - Budget range
//! - Location preferences (with GeoFence)
//! - Property type requirements
//! - Bedroom/bathroom counts
//! - Amenity preferences

use async_trait::async_trait;
use core_models::NodeDef;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::context::ExecutionContext;
use crate::NodeEngineError;
use crate::nodes::NodeHandler;

/// Smart Match Handler - matches leads with suitable properties
pub struct SmartMatchHandler;

#[async_trait]
impl NodeHandler for SmartMatchHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get lead/buyer requirements
        let lead_data = inputs.get("lead").or(inputs.get("record")).cloned()
            .unwrap_or(Value::Null);
        
        // Get properties to match against (from input or config)
        let properties = inputs.get("properties").cloned()
            .unwrap_or(node.config.get("properties").cloned().unwrap_or(Value::Array(vec![])));
        
        // Matching criteria from config
        let budget_weight = node.config.get("budget_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3);
        let location_weight = node.config.get("location_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);
        let type_weight = node.config.get("type_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.2);
        let size_weight = node.config.get("size_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.15);
        let amenity_weight = node.config.get("amenity_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1);
        
        // Extract lead requirements
        let lead_budget_min = lead_data.get("budget_min")
            .or(lead_data.get("min_budget"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let lead_budget_max = lead_data.get("budget_max")
            .or(lead_data.get("max_budget"))
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::MAX);
        let lead_property_type = lead_data.get("property_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let lead_bedrooms_min = lead_data.get("bedrooms_min")
            .or(lead_data.get("min_bedrooms"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let lead_location = lead_data.get("preferred_location")
            .or(lead_data.get("location"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let lead_lat = lead_data.get("latitude")
            .or(lead_data.get("lat"))
            .and_then(|v| v.as_f64());
        let lead_lng = lead_data.get("longitude")
            .or(lead_data.get("lng"))
            .and_then(|v| v.as_f64());
        let lead_radius_km = lead_data.get("radius_km")
            .or(lead_data.get("search_radius"))
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0);
        
        // Score each property
        let mut matches: Vec<Value> = vec![];
        
        if let Some(props) = properties.as_array() {
            for prop in props {
                let mut score = 0.0;
                let mut match_details = HashMap::new();
                
                // Budget match
                let prop_price = prop.get("price")
                    .or(prop.get("asking_price"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                let budget_score = if prop_price >= lead_budget_min && prop_price <= lead_budget_max {
                    1.0
                } else if prop_price < lead_budget_min {
                    // Under budget - still a match but less ideal
                    0.8
                } else {
                    // Over budget - penalize based on how much over
                    let over_percent = (prop_price - lead_budget_max) / lead_budget_max;
                    (1.0 - over_percent).max(0.0)
                };
                score += budget_score * budget_weight;
                match_details.insert("budget_score", json!(budget_score));
                
                // Property type match
                let prop_type = prop.get("property_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let type_score = if lead_property_type.is_empty() || prop_type.to_lowercase() == lead_property_type.to_lowercase() {
                    1.0
                } else {
                    0.0
                };
                score += type_score * type_weight;
                match_details.insert("type_score", json!(type_score));
                
                // Bedroom match
                let prop_bedrooms = prop.get("bedrooms")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let size_score = if prop_bedrooms >= lead_bedrooms_min {
                    1.0
                } else {
                    (prop_bedrooms as f64) / (lead_bedrooms_min as f64).max(1.0)
                };
                score += size_score * size_weight;
                match_details.insert("size_score", json!(size_score));
                
                // Location match (text-based)
                let prop_location = prop.get("city")
                    .or(prop.get("location"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let text_location_score = if lead_location.is_empty() {
                    1.0
                } else if prop_location.to_lowercase().contains(&lead_location.to_lowercase()) {
                    1.0
                } else {
                    0.3
                };
                
                // GeoFence match (if coordinates available)
                let geo_score = match (lead_lat, lead_lng) {
                    (Some(lat1), Some(lng1)) => {
                        let prop_lat = prop.get("latitude").or(prop.get("lat"))
                            .and_then(|v| v.as_f64());
                        let prop_lng = prop.get("longitude").or(prop.get("lng"))
                            .and_then(|v| v.as_f64());
                        
                        if let (Some(lat2), Some(lng2)) = (prop_lat, prop_lng) {
                            let distance = haversine_distance(lat1, lng1, lat2, lng2);
                            if distance <= lead_radius_km {
                                1.0
                            } else {
                                (lead_radius_km / distance).min(1.0)
                            }
                        } else {
                            text_location_score
                        }
                    }
                    _ => text_location_score,
                };
                
                score += geo_score * location_weight;
                match_details.insert("location_score", json!(geo_score));
                
                // Amenity match (simplified)
                let amenity_score = 0.8; // Placeholder - would compare amenity lists
                score += amenity_score * amenity_weight;
                match_details.insert("amenity_score", json!(amenity_score));
                
                // Only include if score meets threshold
                let threshold = node.config.get("threshold")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                
                if score >= threshold {
                    matches.push(json!({
                        "property_id": prop.get("id"),
                        "property": prop,
                        "score": score,
                        "score_percent": (score * 100.0).round(),
                        "match_details": match_details,
                    }));
                }
            }
        }
        
        // Sort by score descending
        matches.sort_by(|a, b| {
            let score_a = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let score_b = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Limit results
        let limit = node.config.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10) as usize;
        matches.truncate(limit);
        
        Ok(json!({
            "matches": matches,
            "total_matched": matches.len(),
            "lead_id": lead_data.get("id"),
        }))
    }
}

/// GeoFence Handler - validates coordinates and calculates distances
pub struct GeoFenceHandler;

#[async_trait]
impl NodeHandler for GeoFenceHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        _context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Get center point from config or input
        let center_lat = node.config.get("latitude")
            .or(inputs.get("center_lat"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing center latitude".into()))?;
        
        let center_lng = node.config.get("longitude")
            .or(inputs.get("center_lng"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing center longitude".into()))?;
        
        let radius_km = node.config.get("radius_km")
            .or(inputs.get("radius"))
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);
        
        // Get point to check
        let check_lat = inputs.get("latitude")
            .or(inputs.get("lat"))
            .and_then(|v| v.as_f64());
        let check_lng = inputs.get("longitude")
            .or(inputs.get("lng"))
            .and_then(|v| v.as_f64());
        
        // If checking a single point
        if let (Some(lat), Some(lng)) = (check_lat, check_lng) {
            let distance = haversine_distance(center_lat, center_lng, lat, lng);
            let is_inside = distance <= radius_km;
            
            return Ok(json!({
                "is_inside": is_inside,
                "distance_km": distance,
                "center": { "lat": center_lat, "lng": center_lng },
                "point": { "lat": lat, "lng": lng },
                "radius_km": radius_km,
            }));
        }
        
        // If checking a list of points
        if let Some(points) = inputs.get("points").and_then(|v| v.as_array()) {
            let results: Vec<Value> = points.iter().map(|point| {
                let lat = point.get("latitude").or(point.get("lat"))
                    .and_then(|v| v.as_f64());
                let lng = point.get("longitude").or(point.get("lng"))
                    .and_then(|v| v.as_f64());
                
                if let (Some(lat), Some(lng)) = (lat, lng) {
                    let distance = haversine_distance(center_lat, center_lng, lat, lng);
                    json!({
                        "id": point.get("id"),
                        "is_inside": distance <= radius_km,
                        "distance_km": distance,
                    })
                } else {
                    json!({
                        "id": point.get("id"),
                        "error": "Missing coordinates",
                    })
                }
            }).collect();
            
            let inside_count = results.iter()
                .filter(|r| r.get("is_inside").and_then(|v| v.as_bool()).unwrap_or(false))
                .count();
            
            return Ok(json!({
                "results": results,
                "inside_count": inside_count,
                "total": points.len(),
            }));
        }
        
        Err(NodeEngineError::InvalidInput("No coordinates provided".into()))
    }
}

/// State Change Handler - handles entity state transitions
pub struct StateChangeHandler;

#[async_trait]
impl NodeHandler for StateChangeHandler {
    async fn execute(
        &self,
        node: &NodeDef,
        inputs: HashMap<String, Value>,
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        let record = inputs.get("record").cloned()
            .unwrap_or(context.trigger_data.clone());
        
        // Get current state
        let state_field = node.config.get("state_field")
            .and_then(|v| v.as_str())
            .unwrap_or("status");
        
        let current_state = record.get(state_field)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Get target state from config
        let target_state = node.config.get("target_state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NodeEngineError::InvalidConfig("Missing target_state".into()))?;
        
        // Get allowed transitions (optional validation)
        let allowed_transitions = node.config.get("allowed_transitions")
            .and_then(|v| v.as_object());
        
        // Validate transition if rules exist
        if let Some(transitions) = allowed_transitions {
            if let Some(allowed_targets) = transitions.get(current_state) {
                if let Some(targets) = allowed_targets.as_array() {
                    let is_allowed = targets.iter()
                        .any(|t| t.as_str() == Some(target_state));
                    
                    if !is_allowed {
                        return Ok(json!({
                            "success": false,
                            "error": format!(
                                "Transition from '{}' to '{}' is not allowed",
                                current_state, target_state
                            ),
                            "current_state": current_state,
                            "target_state": target_state,
                        }));
                    }
                }
            }
        }
        
        // Perform state change (return updated record)
        let mut updated_record = record.clone();
        if let Some(obj) = updated_record.as_object_mut() {
            obj.insert(state_field.to_string(), json!(target_state));
            obj.insert("state_changed_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
            obj.insert("previous_state".to_string(), json!(current_state));
        }
        
        Ok(json!({
            "success": true,
            "previous_state": current_state,
            "new_state": target_state,
            "record": updated_record,
        }))
    }
}

/// Calculate distance between two points using Haversine formula
fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lng = (lng2 - lng1).to_radians();
    
    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    
    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_haversine_distance() {
        // Dubai to Abu Dhabi (~130 km)
        let distance = haversine_distance(25.2048, 55.2708, 24.4539, 54.3773);
        assert!(distance > 120.0 && distance < 140.0);
        
        // Same point
        let distance = haversine_distance(25.0, 55.0, 25.0, 55.0);
        assert!(distance < 0.001);
    }
}
