//! Workflow Editor Page - Visual workflow builder

use leptos::*;
use leptos::html;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;

use crate::api::{fetch_json, API_BASE, TENANT_ID};
use crate::components::workflow::{
    WorkflowCanvas, NodeInspector, NodePalette,
    workflow_canvas::{NodeUI, EdgeUI},
};

/// API Response for workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraphResponse {
    pub workflow_id: Uuid,
    pub name: String,
    pub is_active: bool,
    #[serde(default)]
    pub nodes: Vec<NodeUI>,
    #[serde(default)]
    pub edges: Vec<EdgeUI>,
}

/// Workflow Editor Page component
#[component]
pub fn WorkflowEditorPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    
    let is_new = move || {
        params.with(|p| p.get("id").map(|id| id == "new").unwrap_or(false))
    };
    
    let workflow_id = move || {
        params.with(|p| {
            p.get("id").and_then(|id| {
                if id == "new" { None } else { Uuid::parse_str(id).ok() }
            })
        })
    };

    // State
    let (workflow_name, set_workflow_name) = create_signal("New Workflow".to_string());
    let (is_active, set_is_active) = create_signal(false);
    let (loading, set_loading) = create_signal(true);
    let (saving, set_saving) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (save_message, set_save_message) = create_signal::<Option<String>>(None);
    let (current_workflow_id, set_current_workflow_id) = create_signal::<Option<Uuid>>(None);
    
    // Graph state
    let nodes = create_rw_signal::<Vec<NodeUI>>(vec![]);
    let edges = create_rw_signal::<Vec<EdgeUI>>(vec![]);
    let selected_node_id = create_rw_signal::<Option<Uuid>>(None);
    
    // Derive selected node from ID
    let selected_node = create_memo(move |_| {
        let id = selected_node_id.get();
        nodes.with(|nodes| {
            id.and_then(|id| nodes.iter().find(|n| n.id == id).cloned())
        })
    });

    // Load or create workflow
    create_effect(move |_| {
        if is_new() {
            // New workflow - set up empty canvas with a trigger node
            let trigger_node = NodeUI {
                id: Uuid::new_v4(),
                node_type: "trigger".to_string(),
                label: "Record Created".to_string(),
                x: 200.0,
                y: 200.0,
                config: serde_json::json!({
                    "trigger_type": "record_created",
                    "entity_type": "contact"
                }),
                is_enabled: true,
            };
            nodes.set(vec![trigger_node]);
            edges.set(vec![]);
            set_loading.set(false);
        } else if let Some(id) = workflow_id() {
            set_current_workflow_id.set(Some(id));
            let url = format!("{}/workflows/{}/graph?tenant_id={}", API_BASE, id, TENANT_ID);
            spawn_local(async move {
                match fetch_json::<WorkflowGraphResponse>(&url).await {
                    Ok(data) => {
                        set_workflow_name.set(data.name);
                        set_is_active.set(data.is_active);
                        nodes.set(data.nodes);
                        edges.set(data.edges);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to load workflow: {}", e)));
                        set_loading.set(false);
                    }
                }
            });
        }
    });

    // Save workflow graph
    let save_graph = move |_| {
        set_saving.set(true);
        set_save_message.set(None);
        
        let current_nodes = nodes.get();
        let current_edges = edges.get();
        let name = workflow_name.get();
        let existing_id = current_workflow_id.get();
        
        spawn_local(async move {
            let result = if let Some(id) = existing_id {
                // Update existing workflow
                let url = format!("{}/workflows/{}/graph?tenant_id={}", API_BASE, id, TENANT_ID);
                let body = serde_json::json!({
                    "nodes": current_nodes,
                    "edges": current_edges
                });
                save_workflow_graph(&url, body, "PUT").await
            } else {
                // Create new workflow
                let url = format!("{}/workflows?tenant_id={}", API_BASE, TENANT_ID);
                let body = serde_json::json!({
                    "name": name,
                    "description": "Created via workflow builder",
                    "is_active": false,
                    "nodes": current_nodes,
                    "edges": current_edges
                });
                save_workflow_graph(&url, body, "POST").await
            };
            
            match result {
                Ok(maybe_id) => {
                    set_save_message.set(Some("Saved successfully!".to_string()));
                    set_saving.set(false);
                    if let Some(new_id) = maybe_id {
                        set_current_workflow_id.set(Some(new_id));
                        // Update the URL to the new workflow ID
                        // navigate(&format!("/app/settings/workflows/{}", new_id), Default::default());
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Save failed: {}", e)));
                    set_saving.set(false);
                }
            }
        });
    };

    // Callbacks
    let on_node_select = Callback::new(move |id: Option<Uuid>| {
        selected_node_id.set(id);
    });

    let on_add_node = Callback::new(move |node: NodeUI| {
        nodes.update(|n| n.push(node));
    });

    let on_connect = Callback::new(move |(source, source_port, target, target_port): (Uuid, String, Uuid, String)| {
        let new_edge = EdgeUI {
            id: Uuid::new_v4(),
            source_node: source,
            source_port,
            target_node: target,
            target_port,
        };
        edges.update(|e| e.push(new_edge));
    });

    let on_update_node = Callback::new(move |(id, config): (Uuid, serde_json::Value)| {
        nodes.update(|nodes| {
            if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                node.config = config;
            }
        });
    });

    let on_delete_node = Callback::new(move |id: Uuid| {
        nodes.update(|n| n.retain(|node| node.id != id));
        edges.update(|e| e.retain(|edge| edge.source_node != id && edge.target_node != id));
        selected_node_id.set(None);
    });

    // Keyboard handler for Delete key
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        // Delete or Backspace key deletes selected node
        if ev.key() == "Delete" || ev.key() == "Backspace" {
            // Don't delete if focused on an input element
            if let Some(target) = ev.target() {
                if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                    let tag = element.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" || tag == "select" {
                        return; // Don't intercept when typing in form fields
                    }
                }
            }
            
            if let Some(node_id) = selected_node_id.get() {
                nodes.update(|n| n.retain(|node| node.id != node_id));
                edges.update(|e| e.retain(|edge| edge.source_node != node_id && edge.target_node != node_id));
                selected_node_id.set(None);
                ev.prevent_default();
            }
        }
    };

    // Reference to editor container for auto-focus
    let editor_ref = create_node_ref::<html::Div>();
    
    // Auto-focus editor when it mounts (so Delete key works)
    create_effect(move |_| {
        if let Some(el) = editor_ref.get() {
            let _ = el.focus();
        }
    });

    view! {
        <div 
            class="workflow-editor-page"
            node_ref=editor_ref
            tabindex="0"
            on:keydown=on_keydown
        >
            // Toolbar
            <div class="workflow-toolbar">
                <div class="toolbar-left">
                    <A href="/app/settings/workflows" class="back-btn">
                        "← Back"
                    </A>
                    <h2>{workflow_name}</h2>
                    <span class=move || format!("status-badge {}", if is_active.get() { "active" } else { "inactive" })>
                        {move || if is_active.get() { "Active" } else { "Inactive" }}
                    </span>
                </div>
                <div class="toolbar-right">
                    {move || save_message.get().map(|msg| view! { <span class="save-message">{msg}</span> })}
                    {move || error.get().map(|e| view! { <span class="error-message">{e}</span> })}
                    <button 
                        class="save-btn"
                        disabled=saving
                        on:click=save_graph
                    >
                        {move || if saving.get() { "Saving..." } else { "💾 Save" }}
                    </button>
                </div>
            </div>
            
            // Main editor area
            {move || {
                if loading.get() {
                    view! {
                        <div class="workflow-loading">
                            <div class="loading-spinner"></div>
                            <p>"Loading workflow..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="workflow-editor-layout">
                            // Left sidebar - Node palette
                            <NodePalette on_add_node=on_add_node />
                            
                            // Center - Canvas
                            <div class="workflow-canvas-wrapper">
                                <WorkflowCanvas
                                    nodes=nodes
                                    edges=edges
                                    selected_node=selected_node_id
                                    on_node_select=on_node_select
                                    on_connect=on_connect
                                />
                            </div>
                            
                            // Right sidebar - Inspector
                            <NodeInspector
                                selected_node=Signal::derive(move || selected_node.get())
                                on_update=on_update_node
                                on_delete=on_delete_node
                            />
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Save workflow graph via POST/PUT request
/// Returns the new workflow ID if created via POST
async fn save_workflow_graph(url: &str, body: serde_json::Value, method: &str) -> Result<Option<Uuid>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));
    
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    request.headers().set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;
    
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    
    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "Failed to convert response")?;
    
    if resp.ok() {
        // Try to get created ID from response
        if method == "POST" {
            if let Ok(json_promise) = resp.json() {
                if let Ok(json_value) = JsFuture::from(json_promise).await {
                    // Parse the response to get ID
                    if let Ok(response_data) = serde_wasm_bindgen::from_value::<serde_json::Value>(json_value) {
                        if let Some(id_str) = response_data.get("id").and_then(|v| v.as_str()) {
                            if let Ok(id) = Uuid::parse_str(id_str) {
                                return Ok(Some(id));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    } else {
        Err(format!("HTTP error: {}", resp.status()))
    }
}
