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

    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

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
