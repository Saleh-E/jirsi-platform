//! Workflow Canvas - Infinite pan/zoom canvas with SVG edges and HTML nodes
//!
//! The main visual editor component for workflow graphs.

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::{MouseEvent, WheelEvent};

/// Node UI state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeUI {
    pub id: Uuid,
    pub node_type: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
}

fn default_true() -> bool { true }

/// Edge UI state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeUI {
    pub id: Uuid,
    pub source_node: Uuid,
    pub source_port: String,
    pub target_node: Uuid,
    pub target_port: String,
}

/// Port definition
#[derive(Debug, Clone, PartialEq)]
pub struct PortDef {
    pub name: String,
    pub label: String,
    pub is_input: bool,
}

/// Get ports for a node type
pub fn get_node_ports(node_type: &str) -> (Vec<PortDef>, Vec<PortDef>) {
    let inputs = match node_type {
        "trigger_on_create" | "trigger_on_update" | "trigger_on_delete" => vec![],
        _ => vec![PortDef { name: "input".to_string(), label: "In".to_string(), is_input: true }],
    };

    let outputs = match node_type {
        "condition_if" => vec![
            PortDef { name: "true".to_string(), label: "Yes".to_string(), is_input: false },
            PortDef { name: "false".to_string(), label: "No".to_string(), is_input: false },
        ],
        _ => vec![PortDef { name: "output".to_string(), label: "Out".to_string(), is_input: false }],
    };

    (inputs, outputs)
}

/// Get node color by type
pub fn get_node_color(node_type: &str) -> &'static str {
    if node_type.starts_with("trigger") {
        "#22c55e" // green
    } else if node_type.starts_with("condition") {
        "#eab308" // yellow
    } else if node_type.starts_with("action") {
        "#3b82f6" // blue
    } else if node_type.starts_with("assign") {
        "#a855f7" // purple
    } else if node_type.starts_with("ai") {
        "#f43f5e" // rose
    } else {
        "#6b7280" // gray
    }
}

/// Get node icon by type
pub fn get_node_icon(node_type: &str) -> &'static str {
    match node_type {
        "trigger_on_create" => "⚡",
        "trigger_on_update" => "✏️",
        "trigger_on_delete" => "🗑️",
        "trigger_scheduled" => "⏰",
        "condition_if" => "❓",
        "condition_switch" => "🔀",
        "action_send_email" => "📧",
        "action_create_task" => "✅",
        "action_update_record" => "📝",
        "action_create_record" => "➕",
        "assign_round_robin" => "🔄",
        "assign_load_balanced" => "⚖️",
        "ai_generate" => "✨",
        _ => "⚙️",
    }
}

/// Props for WorkflowCanvas
#[derive(Clone)]
pub struct CanvasProps {
    pub nodes: RwSignal<Vec<NodeUI>>,
    pub edges: RwSignal<Vec<EdgeUI>>,
    pub selected_node: RwSignal<Option<Uuid>>,
    pub on_node_select: Callback<Option<Uuid>>,
    pub on_node_move: Callback<(Uuid, f32, f32)>,
    pub on_connect: Callback<(Uuid, String, Uuid, String)>,
    pub on_delete_edge: Callback<Uuid>,
}

/// WorkflowCanvas component
#[component]
pub fn WorkflowCanvas(
    #[prop(into)] nodes: RwSignal<Vec<NodeUI>>,
    #[prop(into)] edges: RwSignal<Vec<EdgeUI>>,
    #[prop(into)] selected_node: RwSignal<Option<Uuid>>,
    #[prop(into)] on_node_select: Callback<Option<Uuid>>,
    #[prop(into)] on_connect: Callback<(Uuid, String, Uuid, String)>,
) -> impl IntoView {
    // Canvas pan/zoom state
    let (pan_x, set_pan_x) = create_signal(0.0_f32);
    let (pan_y, set_pan_y) = create_signal(0.0_f32);
    let (zoom, set_zoom) = create_signal(1.0_f32);
    
    // Drag state
    let (is_panning, set_is_panning) = create_signal(false);
    let (pan_start_x, set_pan_start_x) = create_signal(0.0_f32);
    let (pan_start_y, set_pan_start_y) = create_signal(0.0_f32);
    
    // Node dragging
    let (dragging_node, set_dragging_node) = create_signal::<Option<Uuid>>(None);
    let (drag_offset_x, set_drag_offset_x) = create_signal(0.0_f32);
    let (drag_offset_y, set_drag_offset_y) = create_signal(0.0_f32);
    
    // Connection drawing
    let (connecting_from, set_connecting_from) = create_signal::<Option<(Uuid, String, f32, f32)>>(None);
    let (mouse_x, set_mouse_x) = create_signal(0.0_f32);
    let (mouse_y, set_mouse_y) = create_signal(0.0_f32);

    // Wheel handler for zoom
    let on_wheel = move |ev: WheelEvent| {
        ev.prevent_default();
        let delta = ev.delta_y() as f32;
        let factor = if delta < 0.0 { 1.1 } else { 0.9 };
        let new_zoom = (zoom.get() * factor).clamp(0.25, 2.0);
        set_zoom.set(new_zoom);
    };

    // Mouse down for pan
    let on_mouse_down = move |ev: MouseEvent| {
        if ev.button() == 1 || (ev.button() == 0 && ev.shift_key()) {
            // Middle click or Shift+Click for pan
            ev.prevent_default();
            set_is_panning.set(true);
            set_pan_start_x.set(ev.client_x() as f32 - pan_x.get());
            set_pan_start_y.set(ev.client_y() as f32 - pan_y.get());
        } else if ev.button() == 0 && connecting_from.get().is_none() && dragging_node.get().is_none() {
            // Click on canvas (not on node) - deselect
            on_node_select.call(None);
        }
    };

    // Mouse move
    let on_mouse_move = move |ev: MouseEvent| {
        set_mouse_x.set(ev.client_x() as f32);
        set_mouse_y.set(ev.client_y() as f32);

        if is_panning.get() {
            set_pan_x.set(ev.client_x() as f32 - pan_start_x.get());
            set_pan_y.set(ev.client_y() as f32 - pan_start_y.get());
        }

        if let Some(node_id) = dragging_node.get() {
            let z = zoom.get();
            let new_x = (ev.client_x() as f32 - pan_x.get()) / z - drag_offset_x.get();
            let new_y = (ev.client_y() as f32 - pan_y.get()) / z - drag_offset_y.get();
            
            nodes.update(|nodes| {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
                    node.x = new_x;
                    node.y = new_y;
                }
            });
        }
    };

    // Mouse up
    let on_mouse_up = move |_ev: MouseEvent| {
        set_is_panning.set(false);
        set_dragging_node.set(None);
        
        // If connecting, cancel
        if connecting_from.get().is_some() {
            set_connecting_from.set(None);
        }
    };

    // Start dragging a node
    let _start_node_drag = move |node_id: Uuid, ev: MouseEvent| {
        ev.stop_propagation();
        let z = zoom.get();
        
        nodes.with(|nodes| {
            if let Some(node) = nodes.iter().find(|n| n.id == node_id) {
                let click_x = (ev.client_x() as f32 - pan_x.get()) / z;
                let click_y = (ev.client_y() as f32 - pan_y.get()) / z;
                set_drag_offset_x.set(click_x - node.x);
                set_drag_offset_y.set(click_y - node.y);
            }
        });
        
        set_dragging_node.set(Some(node_id));
        on_node_select.call(Some(node_id));
    };

    // Start a connection from a port
    let start_connect = move |node_id: Uuid, port: String, x: f32, y: f32| {
        set_connecting_from.set(Some((node_id, port, x, y)));
    };

    // Complete a connection to a port
    let end_connect = {
        let on_connect = on_connect.clone();
        move |target_node: Uuid, target_port: String| {
            if let Some((source_node, source_port, _, _)) = connecting_from.get() {
                if source_node != target_node {
                    on_connect.call((source_node, source_port, target_node, target_port));
                }
            }
            set_connecting_from.set(None);
        }
    };

    // Build SVG edges
    let edges_view = move || {
        edges.get().iter().map(|edge| {
            let source_pos = nodes.with(|nodes| {
                nodes.iter().find(|n| n.id == edge.source_node).map(|n| (n.x + 180.0, n.y + 40.0))
            });
            let target_pos = nodes.with(|nodes| {
                nodes.iter().find(|n| n.id == edge.target_node).map(|n| (n.x, n.y + 40.0))
            });

            if let (Some((sx, sy)), Some((tx, ty))) = (source_pos, target_pos) {
                let mid_x = (sx + tx) / 2.0;
                let path = format!(
                    "M {} {} C {} {} {} {} {} {}",
                    sx, sy, mid_x, sy, mid_x, ty, tx, ty
                );
                view! {
                    <path
                        d=path
                        stroke="#6366f1"
                        stroke-width="2"
                        fill="none"
                        class="workflow-edge"
                    />
                }.into_view()
            } else {
                view! {}.into_view()
            }
        }).collect_view()
    };

    // Build connecting line
    let connecting_line_view = move || {
        if let Some((node_id, _port, _, _)) = connecting_from.get() {
            let source_pos = nodes.with(|nodes| {
                nodes.iter().find(|n| n.id == node_id).map(|n| (n.x + 180.0, n.y + 40.0))
            });
            let z = zoom.get();
            let px = pan_x.get();
            let py = pan_y.get();
            let mx = (mouse_x.get() - px) / z;
            let my = (mouse_y.get() - py) / z;

            if let Some((sx, sy)) = source_pos {
                let mid_x = (sx + mx) / 2.0;
                let path = format!(
                    "M {} {} C {} {} {} {} {} {}",
                    sx, sy, mid_x, sy, mid_x, my, mx, my
                );
                view! {
                    <path
                        d=path
                        stroke="#94a3b8"
                        stroke-width="2"
                        stroke-dasharray="5,5"
                        fill="none"
                    />
                }.into_view()
            } else {
                view! {}.into_view()
            }
        } else {
            view! {}.into_view()
        }
    };

    // Build nodes
    let nodes_view = {
        let on_node_select = on_node_select.clone();
        let start_connect = start_connect.clone();
        let end_connect = end_connect.clone();
        
        move || {
            let on_node_select = on_node_select.clone();
            let start_connect = start_connect.clone();
            let end_connect = end_connect.clone();
            
            nodes.get().iter().map(|node| {
                let node_id = node.id;
                let node_type = node.node_type.clone();
                let label = node.label.clone();
                let x = node.x;
                let y = node.y;
                let is_selected = selected_node.get() == Some(node_id);
                let color = get_node_color(&node_type);
                let icon = get_node_icon(&node_type);
                let (inputs, outputs) = get_node_ports(&node_type);

                let on_port_mouse_down = start_connect.clone();
                let on_port_mouse_up = end_connect.clone();
                
                // Selection and drag handling
                let on_select = on_node_select.clone();
                let on_select2 = on_node_select.clone();

                view! {
                    <div
                        class=format!("workflow-node {}", if is_selected { "selected" } else { "" })
                        style=format!(
                            "left: {}px; top: {}px; border-color: {}; --node-color: {};",
                            x, y, color, color
                        )
                        on:mousedown={
                            move |ev: MouseEvent| {
                                ev.stop_propagation();
                                
                                // Store for dragging
                                let z = zoom.get();
                                nodes.with(|nodes| {
                                    if let Some(node) = nodes.iter().find(|n| n.id == node_id) {
                                        let click_x = (ev.client_x() as f32 - pan_x.get()) / z;
                                        let click_y = (ev.client_y() as f32 - pan_y.get()) / z;
                                        set_drag_offset_x.set(click_x - node.x);
                                        set_drag_offset_y.set(click_y - node.y);
                                    }
                                });
                                
                                set_dragging_node.set(Some(node_id));
                                
                                // Select the node
                                on_select.call(Some(node_id));
                            }
                        }
                        on:click={
                            move |ev: MouseEvent| {
                                ev.stop_propagation();
                                // Also select on click to make sure it works
                                on_select2.call(Some(node_id));
                            }
                        }
                    >
                        <div class="node-header" style=format!("background: {}", color)>
                            <span class="node-icon">{icon}</span>
                            <span class="node-label">{label}</span>
                        </div>
                        <div class="node-body">
                            <div class="node-ports input-ports">
                                {inputs.iter().map(|port| {
                                    let port_name = port.name.clone();
                                    let port_label = port.label.clone();
                                    let on_up = on_port_mouse_up.clone();
                                    view! {
                                        <div 
                                            class="port input-port"
                                            on:mouseup=move |_| on_up(node_id, port_name.clone())
                                        >
                                            <div class="port-dot"></div>
                                            <span class="port-label">{port_label}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="node-ports output-ports">
                                {outputs.iter().map(|port| {
                                    let port_name = port.name.clone();
                                    let port_label = port.label.clone();
                                    let on_down = on_port_mouse_down.clone();
                                    view! {
                                        <div 
                                            class="port output-port"
                                            on:mousedown=move |ev| {
                                                ev.stop_propagation();
                                                on_down(node_id, port_name.clone(), x + 180.0, y + 40.0);
                                            }
                                        >
                                            <span class="port-label">{port_label}</span>
                                            <div class="port-dot"></div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                }
            }).collect_view()
        }
    };


    view! {
        <div
            class="workflow-canvas-container"
            on:wheel=on_wheel
            on:mousedown=on_mouse_down
            on:mousemove=on_mouse_move
            on:mouseup=on_mouse_up
            on:mouseleave=on_mouse_up
        >
            <div
                class="workflow-canvas"
                style=move || format!(
                    "transform: translate({}px, {}px) scale({});",
                    pan_x.get(), pan_y.get(), zoom.get()
                )
            >
                // SVG layer for edges
                <svg class="workflow-edges-svg">
                    {edges_view}
                    {connecting_line_view}
                </svg>
                
                // HTML layer for nodes
                <div class="workflow-nodes">
                    {nodes_view}
                </div>
            </div>
            
            // Zoom controls
            <div class="canvas-controls">
                <button class="zoom-btn" on:click=move |_| set_zoom.update(|z| *z = (*z * 1.2).min(2.0))>
                    "+"
                </button>
                <span class="zoom-level">{move || format!("{:.0}%", zoom.get() * 100.0)}</span>
                <button class="zoom-btn" on:click=move |_| set_zoom.update(|z| *z = (*z / 1.2).max(0.25))>
                    "−"
                </button>
                <button class="zoom-btn" on:click=move |_| { set_pan_x.set(0.0); set_pan_y.set(0.0); set_zoom.set(1.0); }>
                    "⟲"
                </button>
            </div>
            
            // Instructions
            <div class="canvas-help">
                <span>"Scroll to zoom • Shift+drag to pan • Drag nodes to move"</span>
            </div>
        </div>
    }
}
