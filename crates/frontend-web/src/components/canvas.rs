//! Canvas View Component - Relationship visualization
//! Displays entity relationships as a graph with nodes and edges

use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::{fetch_json, API_BASE, TENANT_ID};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipNode {
    pub id: String,
    pub entity_type: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub is_center: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEdge {
    pub from_id: String,
    pub to_id: String,
    pub label: String,
    pub role: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssociationResponse {
    pub id: String,
    pub source_entity_type: String,
    pub source_id: String,
    pub target_entity_type: String,
    pub target_id: String,
    pub target_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssociationsListResponse {
    pub associations: Vec<AssociationResponse>,
}

#[component]
pub fn CanvasView(
    entity_type: String,
    entity_id: String,
    entity_label: String,
) -> impl IntoView {
    let entity_type_stored = store_value(entity_type.clone());
    let entity_id_stored = store_value(entity_id.clone());
    let entity_label_stored = store_value(entity_label.clone());
    
    // State
    let (nodes, set_nodes) = create_signal::<Vec<RelationshipNode>>(Vec::new());
    let (edges, set_edges) = create_signal::<Vec<RelationshipEdge>>(Vec::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (selected_node, set_selected_node) = create_signal::<Option<String>>(None);
    
    // Fetch associations for this entity
    let entity_for_effect = entity_id.clone();
    let etype_for_effect = entity_type.clone();
    create_effect(move |_| {
        let eid = entity_for_effect.clone();
        let et = etype_for_effect.clone();
        let label = entity_label_stored.get_value();
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            let url = format!("{}/associations?tenant_id={}&entity_type={}&entity_id={}", 
                API_BASE, TENANT_ID, et, eid);
            
            match fetch_json::<AssociationsListResponse>(&url).await {
                Ok(response) => {
                    // Build graph from associations
                    let mut graph_nodes = Vec::new();
                    let mut graph_edges = Vec::new();
                    
                    // Add center node (current entity)
                    graph_nodes.push(RelationshipNode {
                        id: eid.clone(),
                        entity_type: et.clone(),
                        label: label.clone(),
                        x: 400.0,
                        y: 300.0,
                        is_center: true,
                    });
                    
                    // Add connected nodes in a circle around center
                    let assoc_count = response.associations.len();
                    for (i, assoc) in response.associations.into_iter().enumerate() {
                        let angle = (i as f64 / assoc_count as f64) * 2.0 * std::f64::consts::PI;
                        let radius = 200.0;
                        let x = 400.0 + radius * angle.cos();
                        let y = 300.0 + radius * angle.sin();
                        
                        let node_label = assoc.target_name.clone()
                            .unwrap_or_else(|| format!("{} #{}", assoc.target_entity_type, &assoc.target_id[..8]));
                        
                        graph_nodes.push(RelationshipNode {
                            id: assoc.target_id.clone(),
                            entity_type: assoc.target_entity_type.clone(),
                            label: node_label,
                            x,
                            y,
                            is_center: false,
                        });
                        
                        graph_edges.push(RelationshipEdge {
                            from_id: eid.clone(),
                            to_id: assoc.target_id,
                            label: assoc.role.clone().unwrap_or_else(|| "linked".to_string()),
                            role: assoc.role,
                        });
                    }
                    
                    set_nodes.set(graph_nodes);
                    set_edges.set(graph_edges);
                    set_loading.set(false);
                }
                Err(e) => {
                    // If no API yet, show placeholder
                    let mut graph_nodes = Vec::new();
                    graph_nodes.push(RelationshipNode {
                        id: eid.clone(),
                        entity_type: et.clone(),
                        label: label.clone(),
                        x: 400.0,
                        y: 300.0,
                        is_center: true,
                    });
                    set_nodes.set(graph_nodes);
                    set_edges.set(Vec::new());
                    set_error.set(Some(format!("No associations found: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Get node color based on entity type
    let get_node_color = |entity_type: &str| -> &'static str {
        match entity_type {
            "property" => "#22c55e",
            "contact" => "#3b82f6",
            "company" => "#8b5cf6",
            "deal" => "#f59e0b",
            "offer" => "#ef4444",
            "contract" => "#06b6d4",
            "viewing" => "#ec4899",
            _ => "#6b7280",
        }
    };
    
    // Get node icon based on entity type
    let get_node_icon = |entity_type: &str| -> &'static str {
        match entity_type {
            "property" => "🏠",
            "contact" => "👤",
            "company" => "🏢",
            "deal" => "💰",
            "offer" => "📝",
            "contract" => "📄",
            "viewing" => "📅",
            _ => "📦",
        }
    };
    
    view! {
        <div class="canvas-container">
            {move || {
                if loading.get() {
                    view! { <div class="canvas-loading">"Loading relationships..."</div> }.into_view()
                } else {
                    view! {
                        <div class="canvas-wrapper">
                            // SVG Canvas for graph
                            <svg class="canvas-svg" viewBox="0 0 800 600" xmlns="http://www.w3.org/2000/svg">
                                // Draw edges first (behind nodes)
                                <For
                                    each=move || edges.get()
                                    key=|e| format!("{}-{}", e.from_id, e.to_id)
                                    children=move |edge| {
                                        let from_node = nodes.get().iter()
                                            .find(|n| n.id == edge.from_id)
                                            .cloned();
                                        let to_node = nodes.get().iter()
                                            .find(|n| n.id == edge.to_id)
                                            .cloned();
                                        
                                        if let (Some(from), Some(to)) = (from_node, to_node) {
                                            let mid_x = (from.x + to.x) / 2.0;
                                            let mid_y = (from.y + to.y) / 2.0;
                                            view! {
                                                <g class="canvas-edge">
                                                    <line 
                                                        x1=from.x 
                                                        y1=from.y 
                                                        x2=to.x 
                                                        y2=to.y
                                                        stroke="#94a3b8"
                                                        stroke-width="2"
                                                    />
                                                    <text 
                                                        x=mid_x 
                                                        y=mid_y - 8.0
                                                        text-anchor="middle"
                                                        class="edge-label"
                                                    >
                                                        {edge.label.clone()}
                                                    </text>
                                                </g>
                                            }.into_view()
                                        } else {
                                            view! {}.into_view()
                                        }
                                    }
                                />
                                
                                // Draw nodes
                                <For
                                    each=move || nodes.get()
                                    key=|n| n.id.clone()
                                    children=move |node| {
                                        let node_id = node.id.clone();
                                        let node_id_for_class = node_id.clone();
                                        let node_id_for_stroke = node_id.clone();
                                        let color = get_node_color(&node.entity_type);
                                        let icon = get_node_icon(&node.entity_type);
                                        let radius = if node.is_center { 50.0 } else { 35.0 };
                                        
                                        view! {
                                            <g 
                                                class=move || {
                                                    let is_selected = selected_node.get() == Some(node_id_for_class.clone());
                                                    format!("canvas-node {}", if is_selected { "selected" } else { "" })
                                                }
                                                on:click={
                                                    let id = node.id.clone();
                                                    move |_| set_selected_node.set(Some(id.clone()))
                                                }
                                            >
                                                <circle
                                                    cx=node.x
                                                    cy=node.y
                                                    r=radius
                                                    fill=color
                                                    stroke=move || {
                                                        let is_selected = selected_node.get() == Some(node_id_for_stroke.clone());
                                                        if is_selected { "white" } else { "transparent" }
                                                    }
                                                    stroke-width="3"
                                                />
                                                <text 
                                                    x=node.x 
                                                    y=node.y + 5.0
                                                    text-anchor="middle"
                                                    class="node-icon"
                                                >
                                                    {icon}
                                                </text>
                                                <text 
                                                    x=node.x 
                                                    y=node.y + radius + 20.0
                                                    text-anchor="middle"
                                                    class="node-label"
                                                >
                                                    {node.label.clone()}
                                                </text>
                                            </g>
                                        }
                                    }
                                />
                            </svg>
                            
                            // Legend
                            <div class="canvas-legend">
                                <h4>"Entity Types"</h4>
                                <div class="legend-item">
                                    <span class="legend-dot" style="background: #22c55e"></span>
                                    "Property"
                                </div>
                                <div class="legend-item">
                                    <span class="legend-dot" style="background: #3b82f6"></span>
                                    "Contact"
                                </div>
                                <div class="legend-item">
                                    <span class="legend-dot" style="background: #8b5cf6"></span>
                                    "Company"
                                </div>
                                <div class="legend-item">
                                    <span class="legend-dot" style="background: #f59e0b"></span>
                                    "Deal"
                                </div>
                            </div>
                            
                            // Info panel for selected node
                            {move || selected_node.get().map(|id| {
                                let selected = nodes.get().iter().find(|n| n.id == id).cloned();
                                selected.map(|node| view! {
                                    <div class="canvas-info-panel">
                                        <h4>{get_node_icon(&node.entity_type)} " " {node.label.clone()}</h4>
                                        <p>"Type: " {node.entity_type.clone()}</p>
                                        <a 
                                            href=format!("/app/crm/entity/{}/{}", node.entity_type, node.id)
                                            class="btn btn-primary"
                                        >
                                            "Open Details"
                                        </a>
                                    </div>
                                })
                            })}
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}
