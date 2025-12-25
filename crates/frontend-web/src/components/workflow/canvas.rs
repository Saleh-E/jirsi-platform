//! Node Graph Canvas - Infinite pan/zoom canvas for visual workflow editor
//!
//! Hybrid Architecture:
//! - SVG Layer: Bezier curve edges (crisp at any zoom)
//! - HTML Layer: Interactive nodes with SmartFields
//! - Synchronized transforms for 60fps performance

use leptos::*;
use leptos::html::Div;
use leptos_use::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

/// Position on the canvas
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Node instance on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInstance {
    pub id: Uuid,
    pub node_type: String,
    pub label: String,
    pub position: Position,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub config: serde_json::Value,
    pub is_selected: bool,
}

/// Port (input or output) on a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub is_required: bool,
}

/// Edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub source_node_id: Uuid,
    pub source_port_id: String,
    pub target_node_id: Uuid,
    pub target_port_id: String,
}

/// Canvas state
#[derive(Debug, Clone, Copy)]
pub struct CanvasState {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

#[component]
pub fn NodeGraphCanvas(
    /// Initial nodes
    #[prop(default = vec![])]
    initial_nodes: Vec<NodeInstance>,
    
    /// Initial edges
    #[prop(default = vec![])]
    initial_edges: Vec<Edge>,
    
    /// Callback when graph changes
    #[prop(optional)]
    on_change: Option<Callback<(Vec<NodeInstance>, Vec<Edge>)>>,
) -> impl IntoView {
    // Canvas state
    let (canvas_state, set_canvas_state) = create_signal(CanvasState::default());
    
    // Nodes and edges
    let (nodes, set_nodes) = create_signal(initial_nodes);
    let (edges, set_edges) = create_signal(initial_edges);
    
    // Interaction state
    let (dragging_node_id, set_dragging_node_id) = create_signal::<Option<Uuid>>(None);
    let (drag_start_pos, set_drag_start_pos) = create_signal::<Option<Position>>(None);
    let (connecting_from, set_connecting_from) = create_signal::<Option<(Uuid, String)>>(None);
    let (connection_preview_end, set_connection_preview_end) = create_signal::<Option<Position>>(None);
    let (is_panning, set_is_panning) = create_signal(false);
    let (pan_start, set_pan_start) = create_signal::<Option<Position>>(None);
    
    // Refs
    let canvas_container_ref = create_node_ref::<Div>();
    
    // ============================================================================
    // PORT CONNECTION HANDLERS
    // ============================================================================
    
    let start_connection = move |node_id: Uuid, port_id: String, ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        set_connecting_from.set(Some((node_id, port_id)));
        set_connection_preview_end.set(Some(Position::new(ev.client_x() as f64, ev.client_y() as f64)));
    };
    
    let finish_connection = move |target_node_id: Uuid, target_port_id: String, _ev: web_sys::MouseEvent| {
        if let Some((source_node_id, source_port_id)) = connecting_from.get() {
            // Don't connect to same node
            if source_node_id != target_node_id {
                // Create new edge
                let new_edge = Edge {
                    id: Uuid::new_v4(),
                    source_node_id,
                    source_port_id,
                    target_node_id,
                    target_port_id,
                };
                
                set_edges.update(|edges_list| {
                    edges_list.push(new_edge);
                });
            }
        }
        
        set_connecting_from.set(None);
        set_connection_preview_end.set(None);
    };
    
    // ============================================================================
    // MOUSE HANDLERS
    // ============================================================================
    
    let on_mouse_down = move |ev: web_sys::MouseEvent| {
        let target = ev.target();
        let target_element = target.and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok());
        
        // Check if clicking on canvas background (for panning)
        if let Some(elem) = target_element {
            if elem.class_list().contains("canvas-background") {
                set_is_panning.set(true);
                set_pan_start.set(Some(Position::new(ev.client_x() as f64, ev.client_y() as f64)));
                ev.prevent_default();
            }
        }
    };
    
    let on_mouse_move = move |ev: web_sys::MouseEvent| {
        let state = canvas_state.get();
        
        // Update connection preview
        if connecting_from.get().is_some() {
            set_connection_preview_end.set(Some(Position::new(ev.client_x() as f64, ev.client_y() as f64)));
        }
        
        // Handle panning
        if is_panning.get() {
            if let Some(start) = pan_start.get() {
                let dx = ev.client_x() as f64 - start.x;
                let dy = ev.client_y() as f64 - start.y;
                
                set_canvas_state.update(|s| {
                    s.pan_x += dx;
                    s.pan_y += dy;
                });
                
                set_pan_start.set(Some(Position::new(ev.client_x() as f64, ev.client_y() as f64)));
            }
        }
        
        // Handle node dragging
        if let Some(node_id) = dragging_node_id.get() {
            if let Some(start) = drag_start_pos.get() {
                let dx = (ev.client_x() as f64 - start.x) / state.zoom;
                let dy = (ev.client_y() as f64 - start.y) / state.zoom;
                
                set_nodes.update(|nodes_list| {
                    if let Some(node) = nodes_list.iter_mut().find(|n| n.id == node_id) {
                        node.position.x += dx;
                        node.position.y += dy;
                    }
                });
                
                set_drag_start_pos.set(Some(Position::new(ev.client_x() as f64, ev.client_y() as f64)));
            }
        }
    };
    
    let on_mouse_up = move |_ev: web_sys::MouseEvent| {
        set_is_panning.set(false);
        set_pan_start.set(None);
        set_dragging_node_id.set(None);
        set_drag_start_pos.set(None);
        set_connecting_from.set(None);
        set_connection_preview_end.set(None);
        
        // Notify parent of changes
        if let Some(callback) = on_change {
            callback.call((nodes.get(), edges.get()));
        }
    };
    
    // ============================================================================
    // ZOOM HANDLER
    // ============================================================================
    
    let on_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        
        let delta = ev.delta_y();
        let zoom_factor = if delta < 0.0 { 1.1 } else { 0.9 };
        
        set_canvas_state.update(|state| {
            state.zoom = (state.zoom * zoom_factor).clamp(0.1, 3.0);
        });
    };
    
    // ============================================================================
    // NODE OPERATIONS
    // ============================================================================
    
    let start_node_drag = move |node_id: Uuid, ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        set_dragging_node_id.set(Some(node_id));
        set_drag_start_pos.set(Some(Position::new(ev.client_x() as f64, ev.client_y() as f64)));
    };
    
    let add_node = move |node_type: String, x: f64, y: f64| {
        let state = canvas_state.get();
        
        // Convert screen coordinates to canvas coordinates
        let canvas_x = (x - state.pan_x) / state.zoom;
        let canvas_y = (y - state.pan_y) / state.zoom;
        
        let new_node = NodeInstance {
            id: Uuid::new_v4(),
            node_type: node_type.clone(),
            label: format!("New {}", node_type),
            position: Position::new(canvas_x, canvas_y),
            inputs: vec![
                Port {
                    id: "input".to_string(),
                    name: "Input".to_string(),
                    data_type: "any".to_string(),
                    is_required: false,
                }
            ],
            outputs: vec![
                Port {
                    id: "output".to_string(),
                    name: "Output".to_string(),
                    data_type: "any".to_string(),
                    is_required: false,
                }
            ],
            config: serde_json::json!({}),
            is_selected: false,
        };
        
        set_nodes.update(|nodes_list| {
            nodes_list.push(new_node);
        });
    };
    
    // ============================================================================
    // RENDER
    // ============================================================================
    
    view! {
        <div
            class="node-graph-canvas"
            on:mousedown=on_mouse_down
            on:mousemove=on_mouse_move
            on:mouseup=on_mouse_up
            on:wheel=on_wheel
            node_ref=canvas_container_ref
        >
            // Canvas background (for panning)
            <div class="canvas-background" />
            
            // SVG layer for edges
            <svg
                class="edges-layer"
                style=move || {
                    let state = canvas_state.get();
                    format!("transform: translate({}px, {}px) scale({})", state.pan_x, state.pan_y, state.zoom)
                }
            >
                <For
                    each=move || edges.get()
                    key=|edge| edge.id
                    children=move |edge| {
                        render_edge(edge, &nodes.get())
                    }
                />
            </svg>
            
            // HTML layer for nodes
            <div
                class="nodes-layer"
                style=move || {
                    let state = canvas_state.get();
                    format!("transform: translate({}px, {}px) scale({})", state.pan_x, state.pan_y, state.zoom)
                }
            >
                <For
                    each=move || nodes.get()
                    key=|node| node.id
                    children=move |node| {
                        render_node(node.clone(), start_node_drag)
                    }
                />
            </div>
            
            // Toolbar
            <div class="canvas-toolbar">
                <button
                    class="toolbar-btn"
                    on:click=move |_| add_node("trigger".to_string(), 100.0, 100.0)
                >
                    <span class="icon">"🎯"</span>
                    " Trigger"
                </button>
                
                <button
                    class="toolbar-btn"
                    on:click=move |_| add_node("action".to_string(), 100.0, 200.0)
                >
                    <span class="icon">"⚡"</span>
                    " Action"
                </button>
                
                <button
                    class="toolbar-btn"
                    on:click=move |_| add_node("condition".to_string(), 100.0, 300.0)
                >
                    <span class="icon">"🔀"</span>
                    " Condition"
                </button>
                
                <button
                    class="toolbar-btn"
                    on:click=move |_| add_node("script".to_string(), 100.0, 400.0)
                >
                    <span class="icon">"📦"</span>
                    " Script"
                </button>
                
                <div class="toolbar-divider" />
                
                // Zoom controls
                <div class="zoom-controls">
                    <button
                        class="zoom-btn"
                        on:click=move |_| {
                            set_canvas_state.update(|s| {
                                s.zoom = (s.zoom * 1.2).min(3.0);
                            });
                        }
                    >
                        "+"
                    </button>
                    
                    <span class="zoom-label">
                        {move || format!("{}%", (canvas_state.get().zoom * 100.0) as i32)}
                    </span>
                    
                    <button
                        class="zoom-btn"
                        on:click=move |_| {
                            set_canvas_state.update(|s| {
                                s.zoom = (s.zoom / 1.2).max(0.1);
                            });
                        }
                    >
                        "-"
                    </button>
                    
                    <button
                        class="zoom-btn"
                        on:click=move |_| {
                            set_canvas_state.set(CanvasState::default());
                        }
                    >
                        "Reset"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Render a single edge as SVG Bezier curve
fn render_edge(edge: Edge, nodes: &[NodeInstance]) -> impl IntoView {
    // Find source and target nodes
    let source_node = nodes.iter().find(|n| n.id == edge.source_node_id);
    let target_node = nodes.iter().find(|n| n.id == edge.target_node_id);
    
    if let (Some(source), Some(target)) = (source_node, target_node) {
        // Calculate edge path
        let start_x = source.position.x + 150.0; // Node width
        let start_y = source.position.y + 50.0;  // Half node height
        let end_x = target.position.x;
        let end_y = target.position.y + 50.0;
        
        // Cubic Bezier curve with horizontal handles
        let control_distance = (end_x - start_x).abs() * 0.5;
        let path = format!(
            "M {} {} C {} {}, {} {}, {} {}",
            start_x, start_y,
            start_x + control_distance, start_y,  // Control point 1
            end_x - control_distance, end_y,      // Control point 2
            end_x, end_y
        );
        
        view! {
            <g class="edge">
                <path
                    d=path.clone()
                    stroke="#6366f1"
                    stroke-width="2"
                    fill="none"
                    class="edge-path"
                />
                // Hit area (wider, invisible)
                <path
                    d=path
                    stroke="transparent"
                    stroke-width="12"
                    fill="none"
                    class="edge-hit-area"
                />
            </g>
        }.into_view()
    } else {
        view! {
            <g />
        }.into_view()
    }
}

/// Render a single node as HTML
fn render_node(
    node: NodeInstance,
    start_drag: impl Fn(Uuid, web_sys::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    let node_id = node.id;
    
    view! {
        <div
            class="node"
            class:selected=node.is_selected
            style=move || format!(
                "left: {}px; top: {}px;",
                node.position.x,
                node.position.y
            )
            on:mousedown=move |ev| start_drag(node_id, ev)
        >
            // Header
            <div class="node-header">
                <span class="node-icon">{get_node_icon(&node.node_type)}</span>
                <span class="node-title">{node.label.clone()}</span>
            </div>
            
            // Input ports
            <div class="node-ports node-inputs">
                {node.inputs.iter().map(|port| {
                    view! {
                        <div class="port input-port" data-port-id=port.id.clone()>
                            <div class="port-dot" />
                            <span class="port-label">{port.name.clone()}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
            
            // Output ports
            <div class="node-ports node-outputs">
                {node.outputs.iter().map(|port| {
                    view! {
                        <div class="port output-port" data-port-id=port.id.clone()>
                            <span class="port-label">{port.name.clone()}</span>
                            <div class="port-dot" />
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Get icon for node type
fn get_node_icon(node_type: &str) -> &'static str {
    match node_type {
        "trigger" => "🎯",
        "action" => "⚡",
        "condition" => "🔀",
        "script" => "📦",
        "ai" => "🤖",
        _ => "📍",
    }
}
