//! Workflow List Page - Shows all workflows with toggle switches

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;

use crate::api::{fetch_json, API_BASE, TENANT_ID};

/// Workflow summary from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub trigger_type: String,
    pub trigger_entity: String,
    pub trigger_count: i32,
    pub last_triggered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response from list endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowListResponse {
    pub workflows: Vec<WorkflowSummary>,
}

/// Toggle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleResponse {
    pub success: bool,
    pub is_active: bool,
}

/// Workflow List Page
#[component]
pub fn WorkflowListPage() -> impl IntoView {
    let (workflows, set_workflows) = create_signal::<Vec<WorkflowSummary>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);

    // Load workflows
    create_effect(move |_| {
        let url = format!("{}/workflows?tenant_id={}", API_BASE, TENANT_ID);
        spawn_local(async move {
            match fetch_json::<WorkflowListResponse>(&url).await {
                Ok(data) => {
                    set_workflows.set(data.workflows);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load workflows: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });

    // Toggle workflow active state
    let toggle_workflow = move |id: Uuid| {
        let url = format!("{}/workflows/{}/toggle?tenant_id={}", API_BASE, id, TENANT_ID);
        
        spawn_local(async move {
            match toggle_workflow_api(&url).await {
                Ok(new_active) => {
                    set_workflows.update(|workflows| {
                        if let Some(wf) = workflows.iter_mut().find(|w| w.id == id) {
                            wf.is_active = new_active;
                        }
                    });
                }
                Err(e) => {
                    set_error.set(Some(format!("Toggle failed: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="workflow-list-page">
            <div class="page-header">
                <div class="header-left">
                    <h1>"Workflow Automation"</h1>
                    <p class="subtitle">"Create and manage automated workflows"</p>
                </div>
                <div class="header-right">
                    <A href="/app/settings/workflows/new" class="btn btn-primary create-workflow-btn">
                        <span class="btn-icon">"+"</span>
                        " Create Workflow"
                    </A>
                    <A href="/app/settings" class="back-link">"← Back to Settings"</A>
                </div>
            </div>

            {move || {
                if loading.get() {
                    view! {
                        <div class="loading-state">
                            <div class="loading-spinner"></div>
                            <p>"Loading workflows..."</p>
                        </div>
                    }.into_view()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="error-state">
                            <p class="error-message">{err}</p>
                        </div>
                    }.into_view()
                } else if workflows.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-icon">"🔄"</div>
                            <h3>"No workflows yet"</h3>
                            <p>"Create your first workflow to automate tasks"</p>
                            <A href="/app/settings/workflows/new" class="btn btn-primary btn-lg">
                                "✨ Create Your First Workflow"
                            </A>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="workflow-list">
                            {move || workflows.get().iter().map(|workflow| {
                                let wf_id = workflow.id;
                                let is_active = workflow.is_active;
                                let toggle = toggle_workflow.clone();
                                
                                view! {
                                    <div class=format!("workflow-card {}", if is_active { "active" } else { "inactive" })>
                                        <div class="workflow-card-header">
                                            <div class="workflow-info">
                                                <h3 class="workflow-name">{workflow.name.clone()}</h3>
                                                {workflow.description.clone().map(|d| view! {
                                                    <p class="workflow-desc">{d}</p>
                                                })}
                                            </div>
                                            <label class="toggle-switch">
                                                <input 
                                                    type="checkbox"
                                                    checked=is_active
                                                    on:change=move |_| toggle(wf_id)
                                                />
                                                <span class="toggle-slider"></span>
                                            </label>
                                        </div>
                                        
                                        <div class="workflow-card-body">
                                            <div class="workflow-meta">
                                                <span class="meta-item">
                                                    <span class="meta-icon">{get_trigger_icon(&workflow.trigger_type)}</span>
                                                    <span class="meta-label">{format_trigger_type(&workflow.trigger_type)}</span>
                                                </span>
                                                <span class="meta-item">
                                                    <span class="meta-icon">"📋"</span>
                                                    <span class="meta-label">{format_entity_type(&workflow.trigger_entity)}</span>
                                                </span>
                                                <span class="meta-item">
                                                    <span class="meta-icon">"▶️"</span>
                                                    <span class="meta-label">{format!("{} runs", workflow.trigger_count)}</span>
                                                </span>
                                            </div>
                                            
                                            {workflow.last_triggered_at.map(|ts| view! {
                                                <div class="last-run">
                                                    "Last run: " {format_timestamp(ts)}
                                                </div>
                                            })}
                                        </div>
                                        
                                        <div class="workflow-card-footer">
                                            <A href=format!("/app/settings/workflows/{}", wf_id) class="edit-btn">
                                                "✏️ Edit Workflow"
                                            </A>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Toggle workflow via PATCH request
async fn toggle_workflow_api(url: &str) -> Result<bool, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let opts = RequestInit::new();
    opts.set_method("PATCH");
    
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    
    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "Failed to convert response")?;
    
    if resp.ok() {
        let json = JsFuture::from(resp.json().map_err(|_| "Failed to get json")?)
            .await
            .map_err(|_| "Failed to parse json")?;
        
        let result: ToggleResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| format!("Failed to deserialize: {:?}", e))?;
        
        Ok(result.is_active)
    } else {
        Err(format!("HTTP error: {}", resp.status()))
    }
}

fn get_trigger_icon(trigger_type: &str) -> &'static str {
    match trigger_type {
        "record_created" => "⚡",
        "record_updated" | "field_changed" => "✏️",
        "record_deleted" => "🗑️",
        "scheduled" => "⏰",
        _ => "🔄",
    }
}

fn format_trigger_type(trigger_type: &str) -> String {
    match trigger_type {
        "record_created" => "On Create".to_string(),
        "record_updated" => "On Update".to_string(),
        "field_changed" => "On Field Change".to_string(),
        "record_deleted" => "On Delete".to_string(),
        "scheduled" => "Scheduled".to_string(),
        _ => trigger_type.replace("_", " "),
    }
}

fn format_entity_type(entity_type: &str) -> String {
    entity_type
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + &entity_type[1..])
        .unwrap_or_else(|| entity_type.to_string())
}

fn format_timestamp(ts: chrono::DateTime<chrono::Utc>) -> String {
    use chrono::Utc;
    
    let now = Utc::now();
    let diff = now.signed_duration_since(ts);
    
    if diff.num_minutes() < 1 {
        "Just now".to_string()
    } else if diff.num_hours() < 1 {
        format!("{} min ago", diff.num_minutes())
    } else if diff.num_days() < 1 {
        format!("{} hours ago", diff.num_hours())
    } else {
        format!("{} days ago", diff.num_days())
    }
}
