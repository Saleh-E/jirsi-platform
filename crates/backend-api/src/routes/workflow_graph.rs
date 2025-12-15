//! Workflow Graph API - CRUD endpoints for visual workflow editor
//!
//! Provides endpoints for listing, retrieving, and saving workflow graphs.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

/// Node definition for the visual editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub config: Value,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
}

fn default_true() -> bool { true }

/// Edge definition for the visual editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source_node: Uuid,
    pub source_port: String,
    pub target_node: Uuid,
    pub target_port: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Complete workflow graph for persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowGraph {
    #[serde(default)]
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

/// Workflow summary for list view
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

/// Full workflow detail including graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDetail {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub trigger_type: String,
    pub trigger_entity: String,
    pub trigger_config: Value,
    pub conditions: Value,
    pub actions: Value,
    pub graph: WorkflowGraph,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new workflow
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_entity: String,
}

/// Request to update workflow graph
#[derive(Debug, Deserialize)]
pub struct UpdateGraphRequest {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Query parameters
#[derive(Debug, Deserialize)]
pub struct WorkflowQuery {
    pub tenant_id: Uuid,
}

/// Create the workflow graph router
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_workflows).post(create_workflow))
        .route("/:id", get(get_workflow))
        .route("/:id/graph", get(get_graph).put(save_graph))
        .route("/:id/toggle", axum::routing::patch(toggle_workflow))
}

/// List all workflows for tenant
async fn list_workflows(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkflowQuery>,
) -> impl IntoResponse {
    let rows: Result<Vec<(Uuid, String, Option<String>, bool, String, String, i32, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>)>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, name, description, is_active, trigger_type, trigger_entity, 
               COALESCE(trigger_count, 0), last_triggered_at, created_at
        FROM workflow_defs
        WHERE tenant_id = $1
        ORDER BY is_system DESC, name ASC
        "#
    )
    .bind(query.tenant_id)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(workflows) => {
            let summaries: Vec<WorkflowSummary> = workflows.into_iter().map(|row| {
                WorkflowSummary {
                    id: row.0,
                    name: row.1,
                    description: row.2,
                    is_active: row.3,
                    trigger_type: row.4,
                    trigger_entity: row.5,
                    trigger_count: row.6,
                    last_triggered_at: row.7,
                    created_at: row.8,
                }
            }).collect();
            
            (StatusCode::OK, Json(json!({ "workflows": summaries })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{}", e) })))
        }
    }
}

/// Get a single workflow with full details
async fn get_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkflowQuery>,
) -> impl IntoResponse {
    let row: Result<Option<(Uuid, String, Option<String>, bool, bool, String, String, Value, Value, Value, Value, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, name, description, is_active, is_system, trigger_type, trigger_entity,
               COALESCE(trigger_config, '{}'::jsonb), COALESCE(conditions, '[]'::jsonb), 
               COALESCE(actions, '[]'::jsonb), COALESCE(node_graph, '{}'::jsonb),
               created_at, updated_at
        FROM workflow_defs
        WHERE id = $1 AND tenant_id = $2
        "#
    )
    .bind(id)
    .bind(query.tenant_id)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(r)) => {
            // Parse node_graph
            let graph: WorkflowGraph = serde_json::from_value(r.10.clone())
                .unwrap_or_default();
            
            let detail = WorkflowDetail {
                id: r.0,
                name: r.1,
                description: r.2,
                is_active: r.3,
                is_system: r.4,
                trigger_type: r.5,
                trigger_entity: r.6,
                trigger_config: r.7,
                conditions: r.8,
                actions: r.9,
                graph,
                created_at: r.11,
                updated_at: r.12,
            };
            
            (StatusCode::OK, Json(json!(detail)))
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "Workflow not found" })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{}", e) })))
        }
    }
}

/// Get just the graph for the canvas
async fn get_graph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkflowQuery>,
) -> impl IntoResponse {
    let row: Result<Option<(Uuid, String, bool, Value)>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, name, is_active, COALESCE(node_graph, '{}'::jsonb)
        FROM workflow_defs
        WHERE id = $1 AND tenant_id = $2
        "#
    )
    .bind(id)
    .bind(query.tenant_id)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some((workflow_id, name, is_active, node_graph))) => {
            let graph: WorkflowGraph = serde_json::from_value(node_graph)
                .unwrap_or_default();
            
            (StatusCode::OK, Json(json!({
                "workflow_id": workflow_id,
                "name": name,
                "is_active": is_active,
                "nodes": graph.nodes,
                "edges": graph.edges
            })))
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "Workflow not found" })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{}", e) })))
        }
    }
}

/// Save the graph from the canvas
async fn save_graph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkflowQuery>,
    Json(request): Json<UpdateGraphRequest>,
) -> impl IntoResponse {
    // Validate the graph
    if let Err(e) = validate_graph(&request.nodes, &request.edges) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e })));
    }

    // Build the node_graph JSON
    let graph = WorkflowGraph {
        nodes: request.nodes,
        edges: request.edges,
    };
    
    let graph_json = serde_json::to_value(&graph).unwrap_or(json!({}));

    // Update the workflow
    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query(
        r#"
        UPDATE workflow_defs 
        SET node_graph = $1, updated_at = NOW()
        WHERE id = $2 AND tenant_id = $3
        "#
    )
    .bind(&graph_json)
    .bind(id)
    .bind(query.tenant_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                (StatusCode::OK, Json(json!({ 
                    "success": true, 
                    "message": "Graph saved successfully"
                })))
            } else {
                (StatusCode::NOT_FOUND, Json(json!({ "error": "Workflow not found" })))
            }
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{}", e) })))
        }
    }
}

/// Toggle workflow active state
async fn toggle_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkflowQuery>,
) -> impl IntoResponse {
    let result: Result<Option<sqlx::postgres::PgRow>, sqlx::Error> = sqlx::query(
        r#"
        UPDATE workflow_defs 
        SET is_active = NOT is_active, updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING is_active
        "#
    )
    .bind(id)
    .bind(query.tenant_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let is_active: bool = row.get("is_active");
            (StatusCode::OK, Json(json!({ 
                "success": true, 
                "is_active": is_active 
            })))
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "Workflow not found" })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{}", e) })))
        }
    }
}

/// Create a new workflow
async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkflowQuery>,
    Json(request): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    let new_id = Uuid::new_v4();
    
    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query(
        r#"
        INSERT INTO workflow_defs (id, tenant_id, name, description, trigger_type, trigger_entity, 
                                   is_active, is_system, conditions, actions, node_graph, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, true, false, '[]'::jsonb, '[]'::jsonb, '{}'::jsonb, NOW(), NOW())
        "#
    )
    .bind(new_id)
    .bind(query.tenant_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.trigger_type)
    .bind(&request.trigger_entity)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            (StatusCode::CREATED, Json(json!({ 
                "id": new_id,
                "name": request.name,
                "success": true 
            })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{}", e) })))
        }
    }
}

/// Validate the workflow graph
fn validate_graph(nodes: &[GraphNode], edges: &[GraphEdge]) -> Result<(), String> {
    if nodes.is_empty() {
        return Err("Graph must have at least one node".to_string());
    }

    // Build node ID set
    let node_ids: HashSet<Uuid> = nodes.iter().map(|n| n.id).collect();

    // Validate all edges reference existing nodes
    for edge in edges {
        if !node_ids.contains(&edge.source_node) {
            return Err(format!("Edge references non-existent source node: {}", edge.source_node));
        }
        if !node_ids.contains(&edge.target_node) {
            return Err(format!("Edge references non-existent target node: {}", edge.target_node));
        }
    }

    // Check for cycles using DFS
    if has_cycle(nodes, edges) {
        return Err("Graph contains a cycle - workflows must be acyclic".to_string());
    }

    // Count trigger nodes
    let trigger_count = nodes.iter()
        .filter(|n| n.node_type.starts_with("trigger"))
        .count();
    
    if trigger_count == 0 {
        return Err("Graph must have at least one trigger node".to_string());
    }

    Ok(())
}

/// Check if the graph has a cycle using DFS
fn has_cycle(nodes: &[GraphNode], edges: &[GraphEdge]) -> bool {
    let node_ids: HashSet<Uuid> = nodes.iter().map(|n| n.id).collect();
    
    // Build adjacency list
    let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for edge in edges {
        adj.entry(edge.source_node).or_default().push(edge.target_node);
    }

    let mut visited: HashSet<Uuid> = HashSet::new();
    let mut rec_stack: HashSet<Uuid> = HashSet::new();

    for node_id in &node_ids {
        if has_cycle_dfs(*node_id, &adj, &mut visited, &mut rec_stack) {
            return true;
        }
    }

    false
}

fn has_cycle_dfs(
    node: Uuid,
    adj: &HashMap<Uuid, Vec<Uuid>>,
    visited: &mut HashSet<Uuid>,
    rec_stack: &mut HashSet<Uuid>,
) -> bool {
    if rec_stack.contains(&node) {
        return true;
    }
    if visited.contains(&node) {
        return false;
    }

    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = adj.get(&node) {
        for neighbor in neighbors {
            if has_cycle_dfs(*neighbor, adj, visited, rec_stack) {
                return true;
            }
        }
    }

    rec_stack.remove(&node);
    false
}
