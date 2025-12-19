//! API service for backend HTTP calls

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

// Demo tenant ID (from seeded data)
pub const TENANT_ID: &str = "b128c8da-6e56-485d-b2fe-e45fb7492b2e";

// Backend API base URL
pub const API_BASE: &str = "http://localhost:3000/api/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub lifecycle_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: String,
    pub name: String,
    pub amount: Option<f64>,
    pub stage: Option<String>,
    pub expected_close_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub reference: String,
    pub title: String,
    pub city: String,
    pub price: Option<i64>,
    pub bedrooms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// Fetch helper for making GET requests
pub async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
    let window = web_sys::window().ok_or("no window")?;

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    request.headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize error: {:?}", e))
}

/// Fetch contacts from API
pub async fn fetch_contacts() -> Result<ListResponse<Contact>, String> {
    let url = format!("{}/entities/contact?tenant_id={}", API_BASE, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch companies from API
pub async fn fetch_companies() -> Result<ListResponse<Company>, String> {
    let url = format!("{}/entities/company?tenant_id={}", API_BASE, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch deals from API
pub async fn fetch_deals() -> Result<ListResponse<Deal>, String> {
    let url = format!("{}/entities/deal?tenant_id={}", API_BASE, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch tasks from API
pub async fn fetch_tasks() -> Result<ListResponse<Task>, String> {
    let url = format!("{}/tasks?tenant_id={}", API_BASE, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch properties from API
pub async fn fetch_properties() -> Result<ListResponse<Property>, String> {
    let url = format!("{}/properties?tenant_id={}", API_BASE, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch counts for dashboard
pub async fn fetch_dashboard_counts() -> Result<DashboardCounts, String> {
    let contacts = fetch_contacts().await.map(|r| r.total).unwrap_or(0);
    let companies = fetch_companies().await.map(|r| r.total).unwrap_or(0);
    let deals = fetch_deals().await.map(|r| r.total).unwrap_or(0);
    let tasks = fetch_tasks().await.map(|r| r.total).unwrap_or(0);
    let properties = fetch_properties().await.map(|r| r.total).unwrap_or(0);

    Ok(DashboardCounts {
        contacts,
        companies,
        deals,
        tasks,
        properties,
    })
}

#[derive(Debug, Clone, Default)]
pub struct DashboardCounts {
    pub contacts: i64,
    pub companies: i64,
    pub deals: i64,
    pub tasks: i64,
    pub properties: i64,
}

// ============================================================================
// METADATA TYPES (for metadata-driven UI)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub id: String,
    pub name: String,
    pub label: String,
    pub label_plural: String,
    pub icon: Option<String>,
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub id: String,
    pub name: String,
    pub label: String,
    pub field_type: serde_json::Value,  // Can be string or object like {"type": "Text"}
    pub is_required: bool,
    #[serde(default)]
    pub show_in_list: bool,
    #[serde(default)]
    pub show_in_card: bool,
    #[serde(default)]
    pub is_readonly: bool,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub help_text: Option<String>,
    #[serde(default)]
    pub ui_hints: Option<serde_json::Value>,
}

impl FieldDef {
    /// Get field type as lowercase string, handling both "Text" and {"type": "Text"} formats
    pub fn get_field_type(&self) -> String {
        if let Some(s) = self.field_type.as_str() {
            s.to_lowercase()
        } else if let Some(obj) = self.field_type.as_object() {
            obj.get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("text")
                .to_lowercase()
        } else {
            "text".to_string()
        }
    }
    
    /// Get options as Vec<(value, label)> for select/status fields
    /// Handles multiple formats: ["val"], [{"value":"v","label":"l"}], {"choices":[...]}
    pub fn get_options(&self) -> Vec<(String, String)> {
        if let Some(ref opts) = self.options {
            // Check for nested "choices" array (some fields use this format)
            if let Some(choices) = opts.get("choices").and_then(|c| c.as_array()) {
                return Self::parse_option_array(choices);
            }
            
            // Direct array format
            if let Some(arr) = opts.as_array() {
                return Self::parse_option_array(arr);
            }
        }
        vec![]
    }
    
    /// Parse an array of options (handles both string and object formats)
    fn parse_option_array(arr: &Vec<serde_json::Value>) -> Vec<(String, String)> {
        arr.iter()
            .filter_map(|item| {
                // Object format: {"value": "x", "label": "X"} or {"value": "x"}
                if let Some(obj) = item.as_object() {
                    let value = obj.get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let label = obj.get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or(value);
                    if !value.is_empty() {
                        return Some((value.to_string(), label.to_string()));
                    }
                }
                // String format: "value"
                if let Some(s) = item.as_str() {
                    return Some((s.to_string(), s.to_string()));
                }
                None
            })
            .collect()
    }
    
    /// Get options as simple strings (backward compatibility)
    pub fn get_option_values(&self) -> Vec<String> {
        self.get_options().into_iter().map(|(v, _)| v).collect()
    }
}

// ============================================================================
// HTTP HELPERS
// ============================================================================

/// POST helper for making POST requests with JSON body (accepts any Serialize type)
pub async fn post_json<B: Serialize, T: for<'de> Deserialize<'de>>(url: &str, body: &B) -> Result<T, String> {
    let window = web_sys::window().ok_or("no window")?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;
    opts.set_body(&JsValue::from_str(&body_str));

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    request.headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize error: {:?}", e))
}

/// PUT helper for making PUT requests with JSON body
pub async fn put_json<T: for<'de> Deserialize<'de>>(url: &str, body: &serde_json::Value) -> Result<T, String> {
    let window = web_sys::window().ok_or("no window")?;

    let opts = RequestInit::new();
    opts.set_method("PUT");
    opts.set_mode(RequestMode::Cors);
    
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;
    opts.set_body(&JsValue::from_str(&body_str));

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    request.headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize error: {:?}", e))
}

/// DELETE helper for making DELETE requests
pub async fn delete_request(url: &str) -> Result<serde_json::Value, String> {
    let window = web_sys::window().ok_or("no window")?;

    let opts = RequestInit::new();
    opts.set_method("DELETE");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize error: {:?}", e))
}

/// PATCH helper for making PATCH requests with generic Serialize body
pub async fn patch_json<B: Serialize, T: for<'de> Deserialize<'de>>(url: &str, body: &B) -> Result<T, String> {
    let window = web_sys::window().ok_or("no window")?;

    let opts = RequestInit::new();
    opts.set_method("PATCH");
    opts.set_mode(RequestMode::Cors);
    
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;
    opts.set_body(&JsValue::from_str(&body_str));

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    request.headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize error: {:?}", e))
}

// ============================================================================
// METADATA API FUNCTIONS
// ============================================================================

/// Fetch all entity types for the tenant
pub async fn fetch_entity_types() -> Result<Vec<EntityType>, String> {
    let url = format!("{}/metadata/entities?tenant_id={}", API_BASE, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch a single entity type by name
pub async fn fetch_entity_type(name: &str) -> Result<EntityType, String> {
    let url = format!("{}/metadata/entities/{}?tenant_id={}", API_BASE, name, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch field definitions for an entity type
pub async fn fetch_field_defs(entity_name: &str) -> Result<Vec<FieldDef>, String> {
    let url = format!("{}/metadata/entities/{}/fields?tenant_id={}", API_BASE, entity_name, TENANT_ID);
    fetch_json(&url).await
}

/// Add a new option to a select/status field's options list
/// This persists the option permanently in the database
pub async fn add_field_option(
    entity_name: &str,
    field_id: &str,
    value: &str,
    label: Option<&str>,
) -> Result<serde_json::Value, String> {
    let url = format!(
        "{}/metadata/entities/{}/fields/{}/options?tenant_id={}", 
        API_BASE, entity_name, field_id, TENANT_ID
    );
    
    let body = serde_json::json!({
        "value": value,
        "label": label.unwrap_or(value)
    });
    
    post_json(&url, &body).await
}

/// Delete an option from a select/status field's options list
/// This permanently removes the option from the database
pub async fn delete_field_option(
    entity_name: &str,
    field_id: &str,
    value: &str,
) -> Result<serde_json::Value, String> {
    let encoded_value = urlencoding::encode(value);
    let url = format!(
        "{}/metadata/entities/{}/fields/{}/options/{}?tenant_id={}", 
        API_BASE, entity_name, field_id, encoded_value, TENANT_ID
    );
    
    delete_request(&url).await
}

// ============================================================================
// LOOKUP API FUNCTIONS (for Link fields)
// ============================================================================

/// Result from entity lookup endpoint - used for Link field dropdowns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupResult {
    pub id: String,
    pub label: String,
}

/// Fetch lookup options for an entity type (for Link field dropdowns)
/// Uses the universal lookup endpoint that returns id/label pairs
pub async fn fetch_entity_lookup(
    entity_type: &str,
    search: Option<&str>,
) -> Result<Vec<LookupResult>, String> {
    let url = match search {
        Some(q) if !q.is_empty() => {
            let encoded_q = urlencoding::encode(q);
            format!("{}/entities/{}/lookup?tenant_id={}&q={}", 
                API_BASE, entity_type, TENANT_ID, encoded_q)
        }
        _ => format!("{}/entities/{}/lookup?tenant_id={}", 
            API_BASE, entity_type, TENANT_ID)
    };
    fetch_json::<Vec<LookupResult>>(&url).await
}

// ============================================================================
// GENERIC ENTITY CRUD FUNCTIONS
// ============================================================================

/// Generic list response for entities
#[derive(Debug, Clone, Deserialize)]
pub struct GenericListResponse {
    pub data: Vec<serde_json::Value>,
    pub total: i64,
}

/// Fetch a list of records for any entity type
pub async fn fetch_entity_list(entity_type: &str) -> Result<GenericListResponse, String> {
    let url = format!("{}/entities/{}?tenant_id={}", API_BASE, entity_type, TENANT_ID);
    fetch_json(&url).await
}

/// Fetch a single record by ID
pub async fn fetch_entity(entity_type: &str, id: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/entities/{}/{}?tenant_id={}", API_BASE, entity_type, id, TENANT_ID);
    fetch_json(&url).await
}

/// Create a new record
pub async fn create_entity(entity_type: &str, data: serde_json::Value) -> Result<serde_json::Value, String> {
    let url = format!("{}/entities/{}?tenant_id={}", API_BASE, entity_type, TENANT_ID);
    post_json(&url, &data).await
}

/// Update an existing record
pub async fn update_entity(entity_type: &str, id: &str, data: serde_json::Value) -> Result<serde_json::Value, String> {
    let url = format!("{}/entities/{}/{}?tenant_id={}", API_BASE, entity_type, id, TENANT_ID);
    put_json(&url, &data).await
}

/// Delete a record (soft delete)
pub async fn delete_entity(entity_type: &str, id: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/entities/{}/{}?tenant_id={}", API_BASE, entity_type, id, TENANT_ID);
    delete_request(&url).await
}

// ============================================================================
// ASSOCIATIONS API FUNCTIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Association {
    pub id: String,
    pub association_def_id: String,
    pub source_id: String,
    pub target_id: String,
    pub role: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationDef {
    pub id: String,
    pub name: String,
    pub source_entity: String,
    pub target_entity: String,
    pub label_source: String,
    pub label_target: String,
    pub cardinality: String,
}

/// Fetch associations for a record (as source or target)
pub async fn fetch_associations(entity_type: &str, record_id: &str) -> Result<Vec<Association>, String> {
    let url = format!(
        "{}/associations?tenant_id={}&source_entity={}&source_id={}",
        API_BASE, TENANT_ID, entity_type, record_id
    );
    fetch_json(&url).await
}

/// Fetch association definitions for an entity type
pub async fn fetch_association_defs(entity_type: &str) -> Result<Vec<AssociationDef>, String> {
    let url = format!(
        "{}/associations/defs?tenant_id={}&source_entity={}",
        API_BASE, TENANT_ID, entity_type
    );
    fetch_json(&url).await
}

/// Create an association between two records
pub async fn create_association(
    association_def_id: &str,
    source_id: &str,
    target_id: &str,
) -> Result<serde_json::Value, String> {
    let url = format!("{}/associations?tenant_id={}", API_BASE, TENANT_ID);
    let body = serde_json::json!({
        "tenant_id": TENANT_ID,
        "association_def_id": association_def_id,
        "source_id": source_id,
        "target_id": target_id,
        "is_primary": false
    });
    post_json(&url, &body).await
}

/// Delete an association
pub async fn delete_association(association_id: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/associations/{}?tenant_id={}", API_BASE, association_id, TENANT_ID);
    delete_request(&url).await
}

// ============================================================================
// INTERACTIONS API FUNCTIONS (Activity Timeline)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub entity_type: String,
    pub record_id: String,
    pub interaction_type: String,
    pub title: String,
    pub content: Option<String>,
    pub created_by: String,
    pub occurred_at: String,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InteractionListResponse {
    pub data: Vec<Interaction>,
    pub total: i64,
}

/// Fetch interactions (activity timeline) for a record
pub async fn fetch_interactions(entity_type: &str, record_id: &str) -> Result<InteractionListResponse, String> {
    let url = format!(
        "{}/interactions?tenant_id={}&entity_type={}&record_id={}",
        API_BASE, TENANT_ID, entity_type, record_id
    );
    fetch_json(&url).await
}

/// Create a new interaction (activity)
pub async fn create_interaction(
    entity_type: &str,
    record_id: &str,
    interaction_type: &str,
    title: &str,
    content: Option<&str>,
    created_by: &str,
) -> Result<serde_json::Value, String> {
    let url = format!("{}/interactions?tenant_id={}", API_BASE, TENANT_ID);
    let body = serde_json::json!({
        "tenant_id": TENANT_ID,
        "entity_type": entity_type,
        "record_id": record_id,
        "interaction_type": interaction_type,
        "title": title,
        "content": content,
        "created_by": created_by
    });
    post_json(&url, &body).await
}

// ============================================================================
// ANALYTICS API FUNCTIONS (Dashboard)
// ============================================================================

/// Dashboard KPI data with targets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardKpis {
    #[serde(default)]
    pub total_leads: i64,
    #[serde(default)]
    pub total_leads_prev: i64,
    #[serde(default)]
    pub leads_trend: f64,
    #[serde(default)]
    pub leads_target: Option<f64>,
    #[serde(default)]
    pub leads_progress: Option<f64>,
    #[serde(default)]
    pub total_deals: i64,
    #[serde(default)]
    pub ongoing_deals: i64,
    #[serde(default)]
    pub total_deals_prev: i64,
    #[serde(default)]
    pub deals_trend: f64,
    #[serde(default)]
    pub deals_target: Option<f64>,
    #[serde(default)]
    pub deals_progress: Option<f64>,
    #[serde(default)]
    pub forecasted_revenue: f64,
    #[serde(default)]
    pub forecasted_revenue_prev: f64,
    #[serde(default)]
    pub revenue_trend: f64,
    #[serde(default)]
    pub revenue_target: Option<f64>,
    #[serde(default)]
    pub revenue_progress: Option<f64>,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub win_rate_prev: f64,
    #[serde(default)]
    pub win_rate_trend: f64,
}

/// Sales trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesTrendPoint {
    pub date: String,
    pub leads: i64,
    pub deals: i64,
}

/// Funnel stage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStage {
    pub stage: String,
    pub count: i64,
}

/// Activity item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: String,
    pub action: String,
    pub entity: String,
    pub entity_name: String,
    pub user: String,
    pub timestamp: String,
}

/// Full dashboard response from API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardResponse {
    #[serde(default)]
    pub kpis: DashboardKpis,
    #[serde(default)]
    pub sales_trend: Vec<SalesTrendPoint>,
    #[serde(default)]
    pub funnel_data: Vec<FunnelStage>,
    #[serde(default)]
    pub recent_activities: Vec<ActivityItem>,
}

/// API wrapper response
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Fetch dashboard data from analytics API with fallback mock data
pub async fn fetch_dashboard(range: &str) -> Result<DashboardResponse, String> {
    let url = format!(
        "http://localhost:3000/api/v1/analytics/dashboard?tenant_id={}&range={}",
        TENANT_ID, range
    );
    
    // Try to fetch from API
    match fetch_json::<ApiResponse<DashboardResponse>>(&url).await {
        Ok(response) => {
            if response.success {
                Ok(response.data.unwrap_or_default())
            } else {
                // API returned error, use mock data
                Ok(mock_dashboard_data(range))
            }
        }
        Err(_) => {
            // Network error - fallback to mock data for development
            Ok(mock_dashboard_data(range))
        }
    }
}

/// Mock dashboard data for development when API is unavailable
/// Returns different data based on range for a realistic demo
fn mock_dashboard_data(range: &str) -> DashboardResponse {
    // Vary the multiplier based on range for realistic effect
    let (leads_mult, deals_mult, trend_dates) = match range {
        "today" => (0.05, 0.02, vec!["9AM", "10AM", "11AM", "12PM", "1PM", "2PM"]),
        "this_week" => (0.3, 0.15, vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]),
        "this_month" => (1.0, 1.0, vec!["Week 1", "Week 2", "Week 3", "Week 4"]),
        "this_quarter" => (3.0, 2.5, vec!["Jan", "Feb", "Mar"]),
        "this_year" => (12.0, 10.0, vec!["Q1", "Q2", "Q3", "Q4"]),
        _ => (1.0, 1.0, vec!["Jan", "Feb", "Mar", "Apr", "May", "Jun"]),
    };
    
    let base_leads = 156.0;
    let base_deals = 28.0;
    let base_revenue = 1_250_000.0;
    
    DashboardResponse {
        kpis: DashboardKpis {
            total_leads: (base_leads * leads_mult) as i64,
            total_leads_prev: (base_leads * leads_mult * 0.88) as i64,
            leads_trend: 13.0,
            leads_target: Some(200.0),
            leads_progress: Some(78.0),
            total_deals: (42.0 * deals_mult) as i64,
            ongoing_deals: (base_deals * deals_mult) as i64,
            total_deals_prev: (38.0 * deals_mult) as i64,
            deals_trend: 10.5,
            deals_target: Some(50.0),
            deals_progress: Some(56.0),
            forecasted_revenue: base_revenue * deals_mult,
            forecasted_revenue_prev: base_revenue * deals_mult * 0.78,
            revenue_trend: 27.5,
            revenue_target: Some(2_000_000.0),
            revenue_progress: Some(62.5),
            win_rate: 68.5,
            win_rate_prev: 62.0,
            win_rate_trend: 6.5,
        },
        sales_trend: trend_dates.iter().enumerate().map(|(i, date)| {
            let base = 40 + (i * 5) as i64;
            SalesTrendPoint { 
                date: date.to_string(), 
                leads: (base as f64 * leads_mult * 0.3) as i64 + 5, 
                deals: (base as f64 * deals_mult * 0.1) as i64 + 2,
            }
        }).collect(),
        funnel_data: vec![
            FunnelStage { stage: "New".to_string(), count: (45.0 * deals_mult) as i64 },
            FunnelStage { stage: "Qualified".to_string(), count: (32.0 * deals_mult) as i64 },
            FunnelStage { stage: "Proposal".to_string(), count: (18.0 * deals_mult) as i64 },
            FunnelStage { stage: "Negotiation".to_string(), count: (12.0 * deals_mult) as i64 },
            FunnelStage { stage: "Won".to_string(), count: (8.0 * deals_mult) as i64 },
        ],
        recent_activities: vec![
            ActivityItem {
                id: "1".to_string(),
                action: "Created".to_string(),
                entity: "Deal".to_string(),
                entity_name: "Luxury Villa Sale".to_string(),
                user: "John Doe".to_string(),
                timestamp: "2 hours ago".to_string(),
            },
            ActivityItem {
                id: "2".to_string(),
                action: "Updated".to_string(),
                entity: "Contact".to_string(),
                entity_name: "Sarah Smith".to_string(),
                user: "Jane Admin".to_string(),
                timestamp: "4 hours ago".to_string(),
            },
            ActivityItem {
                id: "3".to_string(),
                action: "Won".to_string(),
                entity: "Deal".to_string(),
                entity_name: "Downtown Penthouse".to_string(),
                user: "Mike Sales".to_string(),
                timestamp: "Yesterday".to_string(),
            },
        ],
    }
}

// ============================================================================
// INBOX API TYPES AND FUNCTIONS
// ============================================================================

/// An inbox thread (conversation grouped by entity)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InboxThread {
    pub entity_id: String,
    pub entity_type: String,
    pub entity_name: String,
    pub last_message_preview: String,
    pub last_message_at: String,
    pub unread_count: i64,
    pub last_interaction_type: String,
}

/// Thread list response from API
#[derive(Debug, Clone, Deserialize, Default)]
pub struct InboxThreadListResponse {
    pub data: Vec<InboxThread>,
    pub total: i64,
}

/// A message within a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessage {
    pub id: String,
    pub interaction_type: String,
    pub title: String,
    pub content: Option<String>,
    pub created_by: String,
    pub occurred_at: String,
    pub direction: String,
    pub duration_minutes: Option<i32>,
}

/// Messages list response from API
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThreadMessagesResponse {
    pub data: Vec<ThreadMessage>,
    pub entity_name: String,
    pub entity_type: String,
}

/// Wrapper for inbox API data
#[derive(Debug, Clone, Default)]
pub struct ThreadMessagesData {
    pub messages: Vec<ThreadMessage>,
    pub entity_name: String,
    pub entity_type: String,
}

/// Fetch inbox threads with optional filter
pub async fn fetch_inbox_threads(filter: &str) -> Result<Vec<InboxThread>, String> {
    let status_param = match filter {
        "unread" => "&status=unread",
        "sent" => "&status=sent",
        "assigned" => "&assigned_to=me",
        _ => "",
    };
    
    let url = format!(
        "{}/inbox/threads?tenant_id={}{}",
        API_BASE, TENANT_ID, status_param
    );
    
    match fetch_json::<InboxThreadListResponse>(&url).await {
        Ok(response) => Ok(response.data),
        Err(_) => {
            // Return mock data for development
            Ok(mock_inbox_threads())
        }
    }
}

/// Fetch messages for a specific thread
pub async fn fetch_thread_messages(entity_id: &str) -> Result<ThreadMessagesData, String> {
    let url = format!(
        "{}/inbox/threads/{}/messages?tenant_id={}",
        API_BASE, entity_id, TENANT_ID
    );
    
    match fetch_json::<ThreadMessagesResponse>(&url).await {
        Ok(response) => Ok(ThreadMessagesData {
            messages: response.data,
            entity_name: response.entity_name,
            entity_type: response.entity_type,
        }),
        Err(_) => {
            // Return mock data for development
            Ok(mock_thread_messages())
        }
    }
}

/// Send a reply in a thread
pub async fn send_inbox_reply(
    entity_id: &str,
    content: &str,
    interaction_type: &str,
) -> Result<serde_json::Value, String> {
    let url = format!(
        "{}/inbox/threads/{}/reply",
        API_BASE, entity_id
    );
    
    let body = serde_json::json!({
        "tenant_id": TENANT_ID,
        "interaction_type": interaction_type,
        "title": format!("Reply ({})", interaction_type),
        "content": content,
        "created_by": "00000000-0000-0000-0000-000000000000" // TODO: Get from auth context
    });
    
    post_json(&url, &body).await
}

/// Mock inbox threads for development
fn mock_inbox_threads() -> Vec<InboxThread> {
    vec![
        InboxThread {
            entity_id: "11111111-1111-1111-1111-111111111111".to_string(),
            entity_type: "contact".to_string(),
            entity_name: "John Smith".to_string(),
            last_message_preview: "Hi, I'm interested in the downtown property...".to_string(),
            last_message_at: "2024-12-15T10:30:00Z".to_string(),
            unread_count: 2,
            last_interaction_type: "email".to_string(),
        },
        InboxThread {
            entity_id: "22222222-2222-2222-2222-222222222222".to_string(),
            entity_type: "contact".to_string(),
            entity_name: "Jane Doe".to_string(),
            last_message_preview: "Thank you for the viewing yesterday...".to_string(),
            last_message_at: "2024-12-15T09:15:00Z".to_string(),
            unread_count: 0,
            last_interaction_type: "message".to_string(),
        },
        InboxThread {
            entity_id: "33333333-3333-3333-3333-333333333333".to_string(),
            entity_type: "contact".to_string(),
            entity_name: "Bob Johnson".to_string(),
            last_message_preview: "Internal note: Client is interested in financing...".to_string(),
            last_message_at: "2024-12-14T16:00:00Z".to_string(),
            unread_count: 0,
            last_interaction_type: "note".to_string(),
        },
    ]
}

/// Mock thread messages for development
fn mock_thread_messages() -> ThreadMessagesData {
    ThreadMessagesData {
        entity_name: "John Smith".to_string(),
        entity_type: "contact".to_string(),
        messages: vec![
            ThreadMessage {
                id: "msg1".to_string(),
                interaction_type: "email".to_string(),
                title: "Property Inquiry".to_string(),
                content: Some("Hi, I'm interested in the downtown property. Can we schedule a viewing?".to_string()),
                created_by: "client".to_string(),
                occurred_at: "2024-12-15T10:30:00Z".to_string(),
                direction: "inbound".to_string(),
                duration_minutes: None,
            },
            ThreadMessage {
                id: "msg2".to_string(),
                interaction_type: "note".to_string(),
                title: "Internal Note".to_string(),
                content: Some("Client seems very interested. High priority lead.".to_string()),
                created_by: "agent".to_string(),
                occurred_at: "2024-12-15T10:45:00Z".to_string(),
                direction: "outbound".to_string(),
                duration_minutes: None,
            },
            ThreadMessage {
                id: "msg3".to_string(),
                interaction_type: "email".to_string(),
                title: "Re: Property Inquiry".to_string(),
                content: Some("Hello! Thank you for your interest. I'd be happy to arrange a viewing. How does tomorrow at 2pm work for you?".to_string()),
                created_by: "agent".to_string(),
                occurred_at: "2024-12-15T11:00:00Z".to_string(),
                direction: "outbound".to_string(),
                duration_minutes: None,
            },
        ],
    }
}
