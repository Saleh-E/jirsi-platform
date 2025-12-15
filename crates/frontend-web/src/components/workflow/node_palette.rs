//! Node Palette - Draggable catalog of available nodes

use leptos::*;
use uuid::Uuid;

use super::workflow_canvas::{get_node_color, get_node_icon, NodeUI};
use super::workflow_node::{get_node_templates, NodeTemplate};

/// NodePalette component - catalog of available nodes to add
#[component]
pub fn NodePalette(
    #[prop(into)] on_add_node: Callback<NodeUI>,
) -> impl IntoView {
    let categories = get_node_templates();
    let (expanded_category, set_expanded_category) = create_signal::<Option<String>>(Some("Triggers".to_string()));
    
    // Calculate position for new nodes (offset from center)
    let node_count = create_rw_signal(0);
    
    let add_node = move |template: NodeTemplate| {
        let count = node_count.get();
        let new_node = NodeUI {
            id: Uuid::new_v4(),
            node_type: template.node_type,
            label: template.label,
            x: 200.0 + (count as f32 * 20.0),
            y: 150.0 + (count as f32 * 20.0),
            config: serde_json::json!({}),
            is_enabled: true,
        };
        node_count.update(|c| *c += 1);
        on_add_node.call(new_node);
    };

    view! {
        <div class="node-palette">
            <div class="palette-header">
                <h3>"Node Palette"</h3>
            </div>
            <div class="palette-content">
                {categories.into_iter().map(|category| {
                    // Clone everything we need upfront
                    let cat_name_for_check = category.name.clone();
                    let cat_name_for_toggle = category.name.clone();
                    let cat_name_for_display = category.name.clone();
                    let cat_icon = category.icon.clone();
                    
                    view! {
                        <div class="palette-category">
                            <button 
                                class="category-header"
                                on:click={
                                    let toggle_name = cat_name_for_toggle.clone();
                                    let check_name = cat_name_for_check.clone();
                                    move |_| {
                                        let is_expanded = expanded_category.get() == Some(check_name.clone());
                                        if is_expanded {
                                            set_expanded_category.set(None);
                                        } else {
                                            set_expanded_category.set(Some(toggle_name.clone()));
                                        }
                                    }
                                }
                            >
                                <span class="category-icon">{cat_icon.clone()}</span>
                                <span class="category-name">{cat_name_for_display}</span>
                                <span class="category-expand">
                                    {
                                        let check_name = cat_name_for_check.clone();
                                        move || {
                                            if expanded_category.get() == Some(check_name.clone()) {
                                                "▼"
                                            } else {
                                                "▶"
                                            }
                                        }
                                    }
                                </span>
                            </button>
                            <div 
                                class="category-nodes"
                                style={
                                    let check_name = cat_name_for_check.clone();
                                    move || {
                                        if expanded_category.get() == Some(check_name.clone()) {
                                            "display: block;"
                                        } else {
                                            "display: none;"
                                        }
                                    }
                                }
                            >
                                {category.nodes.iter().map(|node_template| {
                                    let template = node_template.clone();
                                    let node_type = template.node_type.clone();
                                    let color = get_node_color(&node_type);
                                    let icon = get_node_icon(&node_type);
                                    let add = add_node.clone();
                                    let label = template.label.clone();
                                    let desc = template.description.clone();
                                    
                                    view! {
                                        <div 
                                            class="palette-node"
                                            on:click={
                                                let template = template.clone();
                                                move |_| add(template.clone())
                                            }
                                        >
                                            <div 
                                                class="palette-node-icon"
                                                style=format!("background: {}", color)
                                            >
                                                {icon}
                                            </div>
                                            <div class="palette-node-info">
                                                <div class="palette-node-label">{label}</div>
                                                <div class="palette-node-desc">{desc}</div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
            
            // Help section
            <div class="palette-help">
                <h4>"Quick Tips"</h4>
                <ul>
                    <li>"Click a node to add it to the canvas"</li>
                    <li>"Drag from output to input to connect"</li>
                    <li>"Select a node to edit properties"</li>
                </ul>
            </div>
        </div>
    }
}
