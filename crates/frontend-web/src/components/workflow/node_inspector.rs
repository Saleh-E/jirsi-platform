//! Node Inspector - Property editor sidebar for selected nodes
//!
//! ## Antigravity Integration
//! Now includes a "Logic" tab for editing visibility and readonly conditions
//! using the Logic Engine (LogicOp).

use leptos::*;
use serde_json::{json, Value};
use uuid::Uuid;

use super::workflow_canvas::NodeUI;

/// Tab selection for the inspector
#[derive(Clone, Copy, PartialEq, Eq)]
enum InspectorTab {
    Properties,
    Logic,
}

/// NodeInspector component - shows properties for selected node
/// Now includes a Logic tab for Antigravity Logic Engine rules
#[component]
pub fn NodeInspector(
    #[prop(into)] selected_node: Signal<Option<NodeUI>>,
    #[prop(into)] on_update: Callback<(Uuid, Value)>,
    #[prop(into)] on_delete: Callback<Uuid>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal(InspectorTab::Properties);
    
    view! {
        <div class="node-inspector">
            {move || {
                if let Some(node) = selected_node.get() {
                    let node_for_tabs = node.clone();
                    let node_for_form = node.clone();
                    let node_for_logic = node.clone();
                    
                    view! {
                        <div class="inspector-content">
                            <div class="inspector-header">
                                <h3>{node.label.clone()}</h3>
                                <button 
                                    class="delete-node-btn"
                                    on:click={
                                        let node_id = node.id;
                                        let on_delete = on_delete.clone();
                                        move |_| on_delete.call(node_id)
                                    }
                                >
                                    "🗑️ Delete"
                                </button>
                            </div>
                            
                            // Tab navigation
                            <div class="inspector-tabs">
                                <button
                                    class="inspector-tab"
                                    class:active=move || active_tab.get() == InspectorTab::Properties
                                    on:click=move |_| set_active_tab.set(InspectorTab::Properties)
                                >
                                    "⚙️ Properties"
                                </button>
                                <button
                                    class="inspector-tab"
                                    class:active=move || active_tab.get() == InspectorTab::Logic
                                    on:click=move |_| set_active_tab.set(InspectorTab::Logic)
                                >
                                    "🔮 Logic"
                                </button>
                            </div>
                            
                            // Tab content
                            <div class="inspector-tab-content">
                                {move || match active_tab.get() {
                                    InspectorTab::Properties => {
                                        view! {
                                            <div class="inspector-section">
                                                <label class="inspector-label">"Node Type"</label>
                                                <div class="inspector-value">{format_node_type(&node_for_tabs.node_type)}</div>
                                            </div>
                                            
                                            <div class="inspector-section">
                                                <label class="inspector-label">"Node ID"</label>
                                                <div class="inspector-value node-id">{node_for_tabs.id.to_string()}</div>
                                            </div>
                                            
                                            // Render dynamic form based on node type
                                            {render_node_form(node_for_form.clone(), on_update.clone())}
                                        }.into_view()
                                    }
                                    InspectorTab::Logic => {
                                        render_logic_tab(node_for_logic.clone(), on_update.clone()).into_view()
                                    }
                                }}
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="inspector-empty">
                            <div class="empty-icon">"📋"</div>
                            <p>"Select a node to edit its properties"</p>
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Render the Logic Tab for Antigravity Logic Engine configuration
/// Now uses LogicBuilder for visual editing with JSON fallback
fn render_logic_tab(node: NodeUI, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let node_id = node.id;
    let config = node.config.clone();
    
    // Extract logic rules from config (or use defaults)
    let visible_if_value = config.get("visible_if")
        .cloned()
        .unwrap_or_else(|| json!("Always"));
    
    let enabled_if_value = config.get("enabled_if")
        .cloned()
        .unwrap_or_else(|| json!("Always"));
    
    let (visible_if, set_visible_if) = create_signal(visible_if_value);
    let (enabled_if, set_enabled_if) = create_signal(enabled_if_value);
    let (show_json_mode, set_show_json_mode) = create_signal(false);
    let (json_error, set_json_error) = create_signal(Option::<String>::None);
    
    // JSON editing signals (for advanced mode)
    let visible_json_str = create_memo(move |_| {
        serde_json::to_string_pretty(&visible_if.get()).unwrap_or_default()
    });
    let enabled_json_str = create_memo(move |_| {
        serde_json::to_string_pretty(&enabled_if.get()).unwrap_or_default()
    });
    
    let (visible_json_edit, set_visible_json_edit) = create_signal(String::new());
    let (enabled_json_edit, set_enabled_json_edit) = create_signal(String::new());
    
    // Sync JSON edit fields when switching to JSON mode
    create_effect(move |_| {
        if show_json_mode.get() {
            set_visible_json_edit.set(visible_json_str.get());
            set_enabled_json_edit.set(enabled_json_str.get());
        }
    });
    
    // Use store_value for config to avoid closure capture issues
    let config_stored = store_value(config.clone());
    let on_update_stored = store_value(on_update.clone());
    
    view! {
        <div class="logic-tab">
            // Header
            <div class="inspector-section">
                <div class="logic-header">
                    <span class="logic-icon">"🔮"</span>
                    <h4>"Antigravity Logic Engine"</h4>
                </div>
                <p class="inspector-help">
                    "Define conditions for when this node should be visible or enabled."
                </p>
                
                // Mode toggle
                <button 
                    class="btn btn-sm btn-outline mode-toggle"
                    on:click=move |_| set_show_json_mode.set(!show_json_mode.get())
                >
                    {move || if show_json_mode.get() { "🎨 Visual Mode" } else { "📝 JSON Mode" }}
                </button>
            </div>
            
            // Show error if any
            {move || json_error.get().map(|err| view! {
                <div class="logic-error">
                    <span class="error-icon">"⚠️"</span>
                    <span>{err}</span>
                </div>
            })}
            
            // Visual Mode
            <Show when=move || !show_json_mode.get()>
                <div class="logic-visual-mode">
                    // Visible If Builder
                    <div class="inspector-section">
                        <label class="inspector-label">"When is this node VISIBLE?"</label>
                        <div class="logic-builder-wrapper">
                            {render_visual_builder(
                                visible_if,
                                "visible".to_string(),
                                move |new_value| {
                                    set_visible_if.set(new_value.clone());
                                    // Update config using stored values
                                    let mut new_config = config_stored.get_value();
                                    if let Some(obj) = new_config.as_object_mut() {
                                        obj.insert("visible_if".to_string(), new_value);
                                        obj.insert("enabled_if".to_string(), enabled_if.get_untracked());
                                    }
                                    on_update_stored.get_value().call((node_id, new_config));
                                }
                            )}
                        </div>
                    </div>
                    
                    // Enabled If Builder
                    <div class="inspector-section">
                        <label class="inspector-label">"When is this node ENABLED?"</label>
                        <div class="logic-builder-wrapper">
                            {render_visual_builder(
                                enabled_if,
                                "enabled".to_string(),
                                move |new_value| {
                                    set_enabled_if.set(new_value.clone());
                                    // Update config using stored values
                                    let mut new_config = config_stored.get_value();
                                    if let Some(obj) = new_config.as_object_mut() {
                                        obj.insert("visible_if".to_string(), visible_if.get_untracked());
                                        obj.insert("enabled_if".to_string(), new_value);
                                    }
                                    on_update_stored.get_value().call((node_id, new_config));
                                }
                            )}
                        </div>
                    </div>
                </div>
            </Show>
            
            // JSON Mode
            <Show when=move || show_json_mode.get()>
                <div class="logic-json-mode">
                    <div class="inspector-section">
                        <label class="inspector-label">"Visible If (JSON)"</label>
                        <textarea 
                            class="inspector-textarea code logic-json"
                            prop:value=visible_json_edit
                            on:input=move |ev| set_visible_json_edit.set(event_target_value(&ev))
                            on:blur=move |_| {
                                match serde_json::from_str::<Value>(&visible_json_edit.get()) {
                                    Ok(parsed) => {
                                        set_json_error.set(None);
                                        set_visible_if.set(parsed.clone());
                                        // Update config using stored values
                                        let mut new_config = config_stored.get_value();
                                        if let Some(obj) = new_config.as_object_mut() {
                                            obj.insert("visible_if".to_string(), parsed);
                                            obj.insert("enabled_if".to_string(), enabled_if.get_untracked());
                                        }
                                        on_update_stored.get_value().call((node_id, new_config));
                                    }
                                    Err(e) => set_json_error.set(Some(format!("Invalid JSON: {}", e))),
                                }
                            }
                        ></textarea>
                    </div>
                    
                    <div class="inspector-section">
                        <label class="inspector-label">"Enabled If (JSON)"</label>
                        <textarea 
                            class="inspector-textarea code logic-json"
                            prop:value=enabled_json_edit
                            on:input=move |ev| set_enabled_json_edit.set(event_target_value(&ev))
                            on:blur=move |_| {
                                match serde_json::from_str::<Value>(&enabled_json_edit.get()) {
                                    Ok(parsed) => {
                                        set_json_error.set(None);
                                        set_enabled_if.set(parsed.clone());
                                        // Update config
                                        let mut new_config = config_stored.get_value();
                                        if let Some(obj) = new_config.as_object_mut() {
                                            obj.insert("visible_if".to_string(), visible_if.get_untracked());
                                            obj.insert("enabled_if".to_string(), parsed);
                                        }
                                        on_update_stored.get_value().call((node_id, new_config));
                                    }
                                    Err(e) => set_json_error.set(Some(format!("Invalid JSON: {}", e))),
                                }
                            }
                        ></textarea>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Render a visual builder for a single LogicOp
fn render_visual_builder<F>(
    value: ReadSignal<Value>,
    id_prefix: String,
    on_change: F,
) -> impl IntoView 
where 
    F: Fn(Value) + Clone + 'static 
{
    let (selected_op, set_selected_op) = create_signal("always".to_string());
    let (role_value, set_role_value) = create_signal(String::new());
    let (field_name, set_field_name) = create_signal(String::new());
    let (field_value, set_field_value) = create_signal(String::new());
    
    // Parse initial value
    create_effect({
        let set_selected_op = set_selected_op.clone();
        let set_role_value = set_role_value.clone();
        let set_field_name = set_field_name.clone();
        let set_field_value = set_field_value.clone();
        move |_| {
            let val = value.get();
            if val.as_str() == Some("Always") || val == json!({"op": "always"}) {
                set_selected_op.set("always".to_string());
            } else if val.as_str() == Some("Never") || val == json!({"op": "never"}) {
                set_selected_op.set("never".to_string());
            } else if let Some(obj) = val.as_object() {
                if let Some(role_obj) = obj.get("HasRole") {
                    set_selected_op.set("hasRole".to_string());
                    if let Some(role) = role_obj.get("role").and_then(|r| r.as_str()) {
                        set_role_value.set(role.to_string());
                    }
                } else if let Some(eq_obj) = obj.get("Equals") {
                    set_selected_op.set("equals".to_string());
                    if let Some(f) = eq_obj.get("field").and_then(|f| f.as_str()) {
                        set_field_name.set(f.to_string());
                    }
                    if let Some(v) = eq_obj.get("value") {
                        set_field_value.set(v.to_string());
                    }
                }
            }
        }
    });
    
    let on_change_op = on_change.clone();
    let on_change_role = on_change.clone();
    let on_change_field = on_change.clone();
    
    let build_value = move || -> Value {
        match selected_op.get().as_str() {
            "never" => json!("Never"),
            "hasRole" => json!({"HasRole": {"role": role_value.get()}}),
            "equals" => {
                let val_str = field_value.get();
                let parsed = serde_json::from_str::<Value>(&val_str)
                    .unwrap_or_else(|_| json!(val_str));
                json!({"Equals": {"field": field_name.get(), "value": parsed}})
            }
            _ => json!("Always"),
        }
    };
    
    view! {
        <div class="visual-logic-builder">
            <select 
                class="inspector-select logic-op-dropdown"
                on:change={
                    let on_change_op = on_change_op.clone();
                    move |ev| {
                        let val = event_target_value(&ev);
                        set_selected_op.set(val);
                        on_change_op(build_value());
                    }
                }
            >
                <option value="always" selected=move || selected_op.get() == "always">"✅ Always"</option>
                <option value="never" selected=move || selected_op.get() == "never">"🚫 Never"</option>
                <option value="hasRole" selected=move || selected_op.get() == "hasRole">"👤 Has Role"</option>
                <option value="equals" selected=move || selected_op.get() == "equals">"🔍 Field Equals"</option>
            </select>
            
            // Role selector (when hasRole is selected)
            <Show when=move || selected_op.get() == "hasRole">
                <div class="logic-param-row">
                    <span class="param-label">"Role:"</span>
                    <select 
                        class="inspector-select param-select"
                        on:change={
                            let on_change_role = on_change_role.clone();
                            move |ev| {
                                set_role_value.set(event_target_value(&ev));
                                on_change_role(build_value());
                            }
                        }
                    >
                        <option value="" selected=move || role_value.get().is_empty()>"Select..."</option>
                        <option value="admin" selected=move || role_value.get() == "admin">"Admin"</option>
                        <option value="manager" selected=move || role_value.get() == "manager">"Manager"</option>
                        <option value="agent" selected=move || role_value.get() == "agent">"Agent"</option>
                        <option value="viewer" selected=move || role_value.get() == "viewer">"Viewer"</option>
                    </select>
                </div>
            </Show>
            
            // Field equals (when equals is selected)
            <Show when=move || selected_op.get() == "equals">
                <div class="logic-param-row">
                    <span class="param-label">"Field:"</span>
                    <input 
                        type="text"
                        class="inspector-input param-input"
                        placeholder="status"
                        prop:value=field_name
                        on:input=move |ev| set_field_name.set(event_target_value(&ev))
                        on:blur={
                            let on_change_field = on_change_field.clone();
                            move |_| on_change_field(build_value())
                        }
                    />
                </div>
                <div class="logic-param-row">
                    <span class="param-label">"Value:"</span>
                    <input 
                        type="text"
                        class="inspector-input param-input"
                        placeholder="active"
                        prop:value=field_value
                        on:input=move |ev| set_field_value.set(event_target_value(&ev))
                        on:blur={
                            let on_change_field = on_change_field.clone();
                            move |_| on_change_field(build_value())
                        }
                    />
                </div>
            </Show>
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

/// Render the form fields for a node based on its type
fn render_node_form(node: NodeUI, on_update: Callback<(Uuid, Value)>) -> View {
    let node_type = node.node_type.as_str();
    let node_id = node.id;
    let config = node.config.clone();

    match node_type {
        // Trigger nodes
        "trigger_on_create" | "trigger_on_update" | "trigger_on_delete" | "trigger_scheduled" => {
            render_trigger_form(node_id, config, on_update).into_view()
        }
        
        // Condition nodes
        "condition_if" => {
            render_condition_form(node_id, config, on_update).into_view()
        }
        
        // Switch condition
        "condition_switch" => {
            render_switch_form(node_id, config, on_update).into_view()
        }
        
        // Email action
        "action_send_email" => {
            render_email_form(node_id, config, on_update).into_view()
        }
        
        // Create task action
        "action_create_task" => {
            render_task_form(node_id, config, on_update).into_view()
        }
        
        // Update/Create record
        "action_update_record" | "action_create_record" => {
            render_record_form(node_id, config, on_update).into_view()
        }
        
        // Assignment nodes
        "assign_round_robin" | "assign_load_balanced" => {
            render_assignment_form(node_id, config, on_update).into_view()
        }

        // AI Generate
        "ai_generate" => {
            render_ai_form(node_id, config, on_update).into_view()
        }
        
        _ => {
            view! {
                <div class="inspector-section">
                    <p class="text-muted">"No additional configuration for this node type."</p>
                </div>
            }.into_view()
        }
    }
}

/// Trigger node form - Entity selection
fn render_trigger_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let entity = config.get("entity").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let (entity_val, set_entity) = create_signal(entity);

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Entity Type"</label>
            <p class="inspector-help">"Which entity type triggers this workflow?"</p>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_entity.set(new_val.clone());
                    let new_config = json!({ "entity": new_val });
                    on_update.call((node_id, new_config));
                }
            >
                <option value="" selected=move || entity_val.get().is_empty()>"Select entity..."</option>
                <option value="contact" selected=move || entity_val.get() == "contact">"Contact"</option>
                <option value="company" selected=move || entity_val.get() == "company">"Company"</option>
                <option value="deal" selected=move || entity_val.get() == "deal">"Deal"</option>
                <option value="property" selected=move || entity_val.get() == "property">"Property"</option>
                <option value="listing" selected=move || entity_val.get() == "listing">"Listing"</option>
                <option value="task" selected=move || entity_val.get() == "task">"Task"</option>
                <option value="viewing" selected=move || entity_val.get() == "viewing">"Viewing"</option>
            </select>
        </div>
    }.into_view()
}

/// Condition IF form - Field comparison
fn render_condition_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let field = config.get("field").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let operator = config.get("operator").and_then(|v| v.as_str()).unwrap_or("equals").to_string();
    let value = config.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let (field_val, set_field) = create_signal(field);
    let (op_val, set_op) = create_signal(operator);
    let (value_val, set_value) = create_signal(value);
    
    let on_update_clone1 = on_update.clone();
    let on_update_clone2 = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Field Name"</label>
            <p class="inspector-help">"The field to check (e.g., status, email, price)"</p>
            <input 
                type="text"
                class="inspector-input"
                placeholder="e.g., status"
                prop:value=field_val
                on:input=move |ev| set_field.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "field": field_val.get(),
                        "operator": op_val.get(),
                        "value": value_val.get()
                    });
                    on_update.call((node_id, new_config));
                }
            />
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Operator"</label>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_op.set(new_val.clone());
                    let new_config = json!({
                        "field": field_val.get(),
                        "operator": new_val,
                        "value": value_val.get()
                    });
                    on_update_clone1.call((node_id, new_config));
                }
            >
                <option value="equals" selected=move || op_val.get() == "equals">"Equals"</option>
                <option value="not_equals" selected=move || op_val.get() == "not_equals">"Not Equals"</option>
                <option value="contains" selected=move || op_val.get() == "contains">"Contains"</option>
                <option value="not_contains" selected=move || op_val.get() == "not_contains">"Not Contains"</option>
                <option value="starts_with" selected=move || op_val.get() == "starts_with">"Starts With"</option>
                <option value="ends_with" selected=move || op_val.get() == "ends_with">"Ends With"</option>
                <option value="greater_than" selected=move || op_val.get() == "greater_than">"Greater Than"</option>
                <option value="less_than" selected=move || op_val.get() == "less_than">"Less Than"</option>
                <option value="is_empty" selected=move || op_val.get() == "is_empty">"Is Empty"</option>
                <option value="is_not_empty" selected=move || op_val.get() == "is_not_empty">"Is Not Empty"</option>
            </select>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Value"</label>
            <p class="inspector-help">"The value to compare against. Use {{field}} for dynamic values."</p>
            <input 
                type="text"
                class="inspector-input"
                placeholder="Value to compare"
                prop:value=value_val
                on:input=move |ev| set_value.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "field": field_val.get(),
                        "operator": op_val.get(),
                        "value": value_val.get()
                    });
                    on_update_clone2.call((node_id, new_config));
                }
            />
        </div>
    }.into_view()
}

/// Switch/Multi-branch condition form
fn render_switch_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let field = config.get("field").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let cases_str = config.get("cases").and_then(|v| v.as_str()).unwrap_or("option1, option2").to_string();
    
    let (field_val, set_field) = create_signal(field);
    let (cases_val, set_cases) = create_signal(cases_str);
    
    let on_update_clone = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Switch Field"</label>
            <p class="inspector-help">"The field whose value determines the branch"</p>
            <input 
                type="text"
                class="inspector-input"
                placeholder="e.g., status"
                prop:value=field_val
                on:input=move |ev| set_field.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "field": field_val.get(),
                        "cases": cases_val.get()
                    });
                    on_update.call((node_id, new_config));
                }
            />
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Case Values"</label>
            <p class="inspector-help">"Comma-separated list of values"</p>
            <input 
                type="text"
                class="inspector-input"
                placeholder="active, pending, closed"
                prop:value=cases_val
                on:input=move |ev| set_cases.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "field": field_val.get(),
                        "cases": cases_val.get()
                    });
                    on_update_clone.call((node_id, new_config));
                }
            />
        </div>
    }.into_view()
}

/// Email action form
fn render_email_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let template = config.get("template").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let to = config.get("to").and_then(|v| v.as_str()).unwrap_or("{{record.email}}").to_string();
    let subject = config.get("subject").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let (template_val, set_template) = create_signal(template);
    let (to_val, set_to) = create_signal(to);
    let (subject_val, set_subject) = create_signal(subject);
    
    let on_update_clone1 = on_update.clone();
    let on_update_clone2 = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Email Template"</label>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_template.set(new_val.clone());
                    let new_config = json!({
                        "template": new_val,
                        "to": to_val.get(),
                        "subject": subject_val.get()
                    });
                    on_update.call((node_id, new_config));
                }
            >
                <option value="" selected=move || template_val.get().is_empty()>"Custom Email"</option>
                <option value="welcome" selected=move || template_val.get() == "welcome">"Welcome Email"</option>
                <option value="follow_up" selected=move || template_val.get() == "follow_up">"Follow Up"</option>
                <option value="reminder" selected=move || template_val.get() == "reminder">"Reminder"</option>
                <option value="notification" selected=move || template_val.get() == "notification">"Notification"</option>
            </select>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"To (Recipient)"</label>
            <p class="inspector-help">"Use {{record.email}} for dynamic recipient"</p>
            <input 
                type="text"
                class="inspector-input"
                placeholder="{{record.email}}"
                prop:value=to_val
                on:input=move |ev| set_to.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "template": template_val.get(),
                        "to": to_val.get(),
                        "subject": subject_val.get()
                    });
                    on_update_clone1.call((node_id, new_config));
                }
            />
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Subject"</label>
            <input 
                type="text"
                class="inspector-input"
                placeholder="Email subject line"
                prop:value=subject_val
                on:input=move |ev| set_subject.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "template": template_val.get(),
                        "to": to_val.get(),
                        "subject": subject_val.get()
                    });
                    on_update_clone2.call((node_id, new_config));
                }
            />
        </div>
    }.into_view()
}

/// Task action form  
fn render_task_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = config.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let assignee = config.get("assignee").and_then(|v| v.as_str()).unwrap_or("owner").to_string();
    let priority = config.get("priority").and_then(|v| v.as_str()).unwrap_or("medium").to_string();
    
    let (title_val, set_title) = create_signal(title);
    let (desc_val, set_desc) = create_signal(description);
    let (assignee_val, set_assignee) = create_signal(assignee);
    let (priority_val, set_priority) = create_signal(priority);
    
    let on_update_clone1 = on_update.clone();
    let on_update_clone2 = on_update.clone();
    let on_update_clone3 = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Task Title"</label>
            <input 
                type="text"
                class="inspector-input"
                placeholder="Follow up with {{record.name}}"
                prop:value=title_val
                on:input=move |ev| set_title.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "title": title_val.get(),
                        "description": desc_val.get(),
                        "assignee": assignee_val.get(),
                        "priority": priority_val.get()
                    });
                    on_update.call((node_id, new_config));
                }
            />
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Description"</label>
            <textarea 
                class="inspector-textarea"
                placeholder="Task description..."
                prop:value=desc_val
                on:input=move |ev| set_desc.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "title": title_val.get(),
                        "description": desc_val.get(),
                        "assignee": assignee_val.get(),
                        "priority": priority_val.get()
                    });
                    on_update_clone1.call((node_id, new_config));
                }
            ></textarea>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Assign To"</label>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_assignee.set(new_val.clone());
                    let new_config = json!({
                        "title": title_val.get(),
                        "description": desc_val.get(),
                        "assignee": new_val,
                        "priority": priority_val.get()
                    });
                    on_update_clone2.call((node_id, new_config));
                }
            >
                <option value="owner" selected=move || assignee_val.get() == "owner">"Record Owner"</option>
                <option value="triggering_user" selected=move || assignee_val.get() == "triggering_user">"Triggering User"</option>
                <option value="assigned_agent" selected=move || assignee_val.get() == "assigned_agent">"Assigned Agent"</option>
                <option value="round_robin" selected=move || assignee_val.get() == "round_robin">"Round Robin (Team)"</option>
            </select>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Priority"</label>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_priority.set(new_val.clone());
                    let new_config = json!({
                        "title": title_val.get(),
                        "description": desc_val.get(),
                        "assignee": assignee_val.get(),
                        "priority": new_val
                    });
                    on_update_clone3.call((node_id, new_config));
                }
            >
                <option value="low" selected=move || priority_val.get() == "low">"Low"</option>
                <option value="medium" selected=move || priority_val.get() == "medium">"Medium"</option>
                <option value="high" selected=move || priority_val.get() == "high">"High"</option>
                <option value="urgent" selected=move || priority_val.get() == "urgent">"Urgent"</option>
            </select>
        </div>
    }.into_view()
}

/// Record update/create form
fn render_record_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let entity = config.get("entity").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let fields_json = config.get("fields").map(|v| serde_json::to_string_pretty(v).unwrap_or_default()).unwrap_or("{}".to_string());
    
    let (entity_val, set_entity) = create_signal(entity);
    let (fields_val, set_fields) = create_signal(fields_json);
    
    let on_update_clone = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Entity Type"</label>
            <p class="inspector-help">"Leave empty to update the triggering record"</p>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_entity.set(new_val.clone());
                    let fields_str = fields_val.get();
                    let fields_parsed: Value = serde_json::from_str(&fields_str).unwrap_or(json!({}));
                    let new_config = json!({
                        "entity": new_val,
                        "fields": fields_parsed
                    });
                    on_update.call((node_id, new_config));
                }
            >
                <option value="" selected=move || entity_val.get().is_empty()>"Same as Trigger"</option>
                <option value="contact" selected=move || entity_val.get() == "contact">"Contact"</option>
                <option value="company" selected=move || entity_val.get() == "company">"Company"</option>
                <option value="deal" selected=move || entity_val.get() == "deal">"Deal"</option>
                <option value="task" selected=move || entity_val.get() == "task">"Task"</option>
                <option value="property" selected=move || entity_val.get() == "property">"Property"</option>
            </select>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Field Updates (JSON)"</label>
            <p class="inspector-help">"JSON object with field names and values"</p>
            <textarea 
                class="inspector-textarea code"
                placeholder=r#"{"status": "active"}"#
                prop:value=fields_val
                on:input=move |ev| set_fields.set(event_target_value(&ev))
                on:blur=move |_| {
                    let fields_str = fields_val.get();
                    let fields_parsed: Value = serde_json::from_str(&fields_str).unwrap_or(json!({}));
                    let new_config = json!({
                        "entity": entity_val.get(),
                        "fields": fields_parsed
                    });
                    on_update_clone.call((node_id, new_config));
                }
            ></textarea>
        </div>
    }.into_view()
}

/// Assignment node form
fn render_assignment_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let strategy = config.get("strategy").and_then(|v| v.as_str()).unwrap_or("round_robin").to_string();
    let field = config.get("assign_to_field").and_then(|v| v.as_str()).unwrap_or("owner_id").to_string();
    let team = config.get("team").and_then(|v| v.as_str()).unwrap_or("all").to_string();
    
    let (strategy_val, set_strategy) = create_signal(strategy);
    let (field_val, set_field) = create_signal(field);
    let (team_val, set_team) = create_signal(team);
    
    let on_update_clone1 = on_update.clone();
    let on_update_clone2 = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"Assignment Strategy"</label>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_strategy.set(new_val.clone());
                    let new_config = json!({
                        "strategy": new_val,
                        "assign_to_field": field_val.get(),
                        "team": team_val.get()
                    });
                    on_update.call((node_id, new_config));
                }
            >
                <option value="round_robin" selected=move || strategy_val.get() == "round_robin">"Round Robin"</option>
                <option value="load_balanced" selected=move || strategy_val.get() == "load_balanced">"Load Balanced"</option>
                <option value="random" selected=move || strategy_val.get() == "random">"Random"</option>
            </select>
            <p class="inspector-help">
                {move || {
                    match strategy_val.get().as_str() {
                        "round_robin" => "Assigns to agents in rotation",
                        "load_balanced" => "Assigns to agent with fewest deals",
                        "random" => "Random assignment",
                        _ => ""
                    }
                }}
            </p>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Assign To Field"</label>
            <p class="inspector-help">"The field on the record to update"</p>
            <input 
                type="text"
                class="inspector-input"
                placeholder="owner_id"
                prop:value=field_val
                on:input=move |ev| set_field.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "strategy": strategy_val.get(),
                        "assign_to_field": field_val.get(),
                        "team": team_val.get()
                    });
                    on_update_clone1.call((node_id, new_config));
                }
            />
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"Team"</label>
            <select 
                class="inspector-select"
                on:change=move |ev| {
                    let new_val = event_target_value(&ev);
                    set_team.set(new_val.clone());
                    let new_config = json!({
                        "strategy": strategy_val.get(),
                        "assign_to_field": field_val.get(),
                        "team": new_val
                    });
                    on_update_clone2.call((node_id, new_config));
                }
            >
                <option value="all" selected=move || team_val.get() == "all">"All Agents"</option>
                <option value="sales" selected=move || team_val.get() == "sales">"Sales Team"</option>
                <option value="support" selected=move || team_val.get() == "support">"Support Team"</option>
                <option value="managers" selected=move || team_val.get() == "managers">"Managers Only"</option>
            </select>
        </div>
    }.into_view()
}

/// AI Generate form
fn render_ai_form(node_id: Uuid, config: Value, on_update: Callback<(Uuid, Value)>) -> impl IntoView {
    let prompt = config.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let system_prompt = config.get("system_prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let (prompt_val, set_prompt) = create_signal(prompt);
    let (sys_val, set_sys) = create_signal(system_prompt);
    
    let on_update_clone = on_update.clone();

    view! {
        <div class="inspector-section">
            <label class="inspector-label">"System Prompt"</label>
            <p class="inspector-help">"Instructions for the AI assistant"</p>
             <textarea 
                class="inspector-textarea"
                placeholder="You are a helpful CRM assistant..."
                prop:value=sys_val
                on:input=move |ev| set_sys.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "prompt": prompt_val.get(),
                        "system_prompt": sys_val.get()
                    });
                    on_update.call((node_id, new_config));
                }
            ></textarea>
        </div>
        <div class="inspector-section">
            <label class="inspector-label">"User Prompt Template"</label>
            <p class="inspector-help">"Use {{variable}} to insert values"</p>
            <textarea 
                class="inspector-textarea code"
                placeholder="Summarize this email: {{email_body}}"
                prop:value=prompt_val
                on:input=move |ev| set_prompt.set(event_target_value(&ev))
                on:blur=move |_| {
                    let new_config = json!({
                        "prompt": prompt_val.get(),
                        "system_prompt": sys_val.get()
                    });
                    on_update_clone.call((node_id, new_config));
                }
            ></textarea>
        </div>
    }.into_view()
}
