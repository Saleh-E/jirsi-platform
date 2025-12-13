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
    pub field_type: String,
    pub is_required: bool,
    pub show_in_list: bool,
    pub show_in_card: bool,
    pub is_readonly: bool,
    pub sort_order: i32,
    pub options: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

// ============================================================================
// HTTP HELPERS
// ============================================================================

/// POST helper for making POST requests with JSON body
pub async fn post_json<T: for<'de> Deserialize<'de>>(url: &str, body: &serde_json::Value) -> Result<T, String> {
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
