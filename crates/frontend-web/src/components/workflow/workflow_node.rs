//! Workflow Node - Individual node component with ports

use leptos::*;
use uuid::Uuid;

use super::workflow_canvas::{get_node_color, get_node_icon, get_node_ports, NodeUI};

/// Individual workflow node component
#[component]
pub fn WorkflowNode(
    #[prop(into)] node: NodeUI,
    #[prop(into)] is_selected: bool,
    #[prop(into)] on_select: Callback<Uuid>,
    #[prop(into)] on_start_connect: Callback<(Uuid, String)>,
    #[prop(into)] on_end_connect: Callback<(Uuid, String)>,
) -> impl IntoView {
    let node_id = node.id;
    let node_type = node.node_type.clone();
    let color = get_node_color(&node.node_type);
    let icon = get_node_icon(&node.node_type);
    let (inputs, outputs) = get_node_ports(&node.node_type);

    view! {
        <div
            class=format!("workflow-node {}", if is_selected { "selected" } else { "" })
            style=format!(
                "left: {}px; top: {}px; border-color: {}; --node-color: {};",
                node.x, node.y, color, color
            )
            on:click=move |ev| {
                ev.stop_propagation();
                on_select.call(node_id);
            }
        >
            <div class="node-header" style=format!("background: {}", color)>
                <span class="node-icon">{icon}</span>
                <span class="node-label">{node.label.clone()}</span>
                {if !node.is_enabled {
                    view! { <span class="node-disabled-badge">"Disabled"</span> }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </div>
            <div class="node-body">
                // Input ports (left side)
                <div class="node-ports input-ports">
                    {inputs.iter().map(|port| {
                        let port_name = port.name.clone();
                        let port_label = port.label.clone();
                        let on_end = on_end_connect.clone();
                        view! {
                            <div 
                                class="port input-port"
                                on:mouseup=move |ev| {
                                    ev.stop_propagation();
                                    on_end.call((node_id, port_name.clone()));
                                }
                            >
                                <div class="port-dot input-dot"></div>
                                <span class="port-label">{port_label}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
                
                // Node type label
                <div class="node-type-label">
                    {format_node_type(&node_type)}
                </div>
                
                // Output ports (right side)
                <div class="node-ports output-ports">
                    {outputs.iter().map(|port| {
                        let port_name = port.name.clone();
                        let port_label = port.label.clone();
                        let on_start = on_start_connect.clone();
                        view! {
                            <div 
                                class="port output-port"
                                on:mousedown=move |ev| {
                                    ev.stop_propagation();
                                    ev.prevent_default();
                                    on_start.call((node_id, port_name.clone()));
                                }
                            >
                                <span class="port-label">{port_label}</span>
                                <div class="port-dot output-dot"></div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

/// Format node type for display
fn format_node_type(node_type: &str) -> String {
    node_type
        .replace("_", " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Node type categories for the palette
#[derive(Debug, Clone)]
pub struct NodeCategory {
    pub name: String,
    pub icon: String,
    pub nodes: Vec<NodeTemplate>,
}

/// Template for creating new nodes
#[derive(Debug, Clone)]
pub struct NodeTemplate {
    pub node_type: String,
    pub label: String,
    pub description: String,
}

/// Get all available node templates organized by category
pub fn get_node_templates() -> Vec<NodeCategory> {
    vec![
        NodeCategory {
            name: "Triggers".to_string(),
            icon: "⚡".to_string(),
            nodes: vec![
                NodeTemplate {
                    node_type: "trigger_on_create".to_string(),
                    label: "On Record Created".to_string(),
                    description: "Fires when a new record is created".to_string(),
                },
                NodeTemplate {
                    node_type: "trigger_on_update".to_string(),
                    label: "On Record Updated".to_string(),
                    description: "Fires when a record is modified".to_string(),
                },
                NodeTemplate {
                    node_type: "trigger_on_delete".to_string(),
                    label: "On Record Deleted".to_string(),
                    description: "Fires when a record is deleted".to_string(),
                },
                NodeTemplate {
                    node_type: "trigger_scheduled".to_string(),
                    label: "Scheduled".to_string(),
                    description: "Fires on a schedule (daily, weekly)".to_string(),
                },
            ],
        },
        NodeCategory {
            name: "Conditions".to_string(),
            icon: "❓".to_string(),
            nodes: vec![
                NodeTemplate {
                    node_type: "condition_if".to_string(),
                    label: "If / Else".to_string(),
                    description: "Branch based on a condition".to_string(),
                },
                NodeTemplate {
                    node_type: "condition_switch".to_string(),
                    label: "Switch".to_string(),
                    description: "Branch based on multiple values".to_string(),
                },
            ],
        },
        NodeCategory {
            name: "Actions".to_string(),
            icon: "▶️".to_string(),
            nodes: vec![
                NodeTemplate {
                    node_type: "action_send_email".to_string(),
                    label: "Send Email".to_string(),
                    description: "Send an email notification".to_string(),
                },
                NodeTemplate {
                    node_type: "action_create_task".to_string(),
                    label: "Create Task".to_string(),
                    description: "Create a new task".to_string(),
                },
                NodeTemplate {
                    node_type: "action_update_record".to_string(),
                    label: "Update Record".to_string(),
                    description: "Update fields on a record".to_string(),
                },
                NodeTemplate {
                    node_type: "action_create_record".to_string(),
                    label: "Create Record".to_string(),
                    description: "Create a new record".to_string(),
                },
            ],
        },
        NodeCategory {
            name: "Assignment".to_string(),
            icon: "👥".to_string(),
            nodes: vec![
                NodeTemplate {
                    node_type: "assign_round_robin".to_string(),
                    label: "Round Robin".to_string(),
                    description: "Assign to agents in rotation".to_string(),
                },
                NodeTemplate {
                    node_type: "assign_load_balanced".to_string(),
                    label: "Load Balanced".to_string(),
                    description: "Assign to agent with fewest deals".to_string(),
                },
            ],
        },
        NodeCategory {
            name: "AI & Agents".to_string(),
            icon: "🤖".to_string(),
            nodes: vec![
                NodeTemplate {
                    node_type: "ai_generate".to_string(),
                    label: "AI Generate".to_string(),
                    description: "Generate text using LLM".to_string(),
                },
            ],
        },
    ]
}
