//! Workflow Pipe - Batch process records through a workflow
//!
//! Allows users to select rows in a table and run them through
//! a workflow for automated batch processing.

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub workflow_id: Uuid,
    pub total_records: usize,
    pub processed: usize,
    pub failed: usize,
    pub execution_time_ms: u64,
}

#[component]
pub fn WorkflowPipe(
    /// Selected record IDs to process
    selected_ids: Signal<Vec<Uuid>>,
    
    /// Entity type being processed
    entity_type: String,
    
    /// Callback when execution completes
    #[prop(optional)]
    on_complete: Option<Callback<ExecutionResult>>,
) -> impl IntoView {
    // State
    let (show_modal, set_show_modal) = create_signal(false);
    let (workflows, set_workflows) = create_signal::<Vec<WorkflowDef>>(vec![]);
    let (executing, set_executing) = create_signal(false);
    let (progress, set_progress) = create_signal(0);
    let (current_workflow, set_current_workflow) = create_signal::<Option<WorkflowDef>>(None);
    
    // Fetch available workflows for this entity type
    create_effect(move |_| {
        if show_modal.get() {
            spawn_local(async move {
                // TODO: Real API call
                let mock_workflows = vec![
                    WorkflowDef {
                        id: Uuid::new_v4(),
                        name: "Send Welcome Email".to_string(),
                        description: Some("Sends welcome email to new contacts".to_string()),
                        is_enabled: true,
                    },
                    WorkflowDef {
                        id: Uuid::new_v4(),
                        name: "Update Status".to_string(),
                        description: Some("Updates contact status to 'Active'".to_string()),
                        is_enabled: true,
                    },
                    WorkflowDef {
                        id: Uuid::new_v4(),
                        name: "Generate Report".to_string(),
                        description: Some("Generates summary report".to_string()),
                        is_enabled: true,
                    },
                ];
                
                set_workflows.set(mock_workflows);
            });
        }
    });
    
    // Execute workflow
    let execute_workflow = move |workflow: WorkflowDef| {
        set_executing.set(true);
        set_current_workflow.set(Some(workflow.clone()));
        set_progress.set(0);
        
        spawn_local(async move {
            let total = selected_ids.get().len();
            
            // Simulate processing
            for i in 0..total {
                set_progress.set(((i + 1) as f64 / total as f64 * 100.0) as i32);
                
                // Simulate API call
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
            
            set_executing.set(false);
            set_show_modal.set(false);
            set_progress.set(0);
            
            // Call completion callback
            if let Some(callback) = on_complete {
                callback.call(ExecutionResult {
                    workflow_id: workflow.id,
                    total_records: total,
                    processed: total,
                    failed: 0,
                    execution_time_ms: (total as u64) * 100,
                });
            }
        });
    };
    
    view! {
        <div class="workflow-pipe">
            // Trigger button
            <button
                class="btn btn-primary workflow-pipe-btn"
                disabled=move || selected_ids.get().is_empty()
                on:click=move |_| set_show_modal.set(true)
            >
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path d="M5 3l14 9-14 9V3z" />
                </svg>
                <span>"Run Workflow"</span>
                <span class="badge">
                    {move || selected_ids.get().len()}
                </span>
            </button>
            
            // Workflow selection modal
            <Show when=move || show_modal.get()>
                <div class="modal-overlay" on:click=move |_| {
                    if !executing.get() {
                        set_show_modal.set(false);
                    }
                }>
                    <div class="modal-content workflow-modal" on:click=|ev| ev.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Select Workflow"</h2>
                            <button
                                class="modal-close"
                                disabled=executing
                                on:click=move |_| set_show_modal.set(false)
                            >
                                "×"
                            </button>
                        </div>
                        
                        <div class="modal-body">
                            <p class="workflow-info">
                                <strong>{move || selected_ids.get().len()}</strong>
                                " records selected for processing"
                            </p>
                            
                            <Show
                                when=move || !executing.get()
                                fallback=move || view! {
                                    <div class="workflow-executing">
                                        <div class="execution-header">
                                            <div class="spinner" />
                                            <h3>"Executing Workflow"</h3>
                                        </div>
                                        
                                        {move || current_workflow.get().map(|wf| view! {
                                            <p class="workflow-name">{wf.name}</p>
                                        })}
                                        
                                        <div class="progress-bar">
                                            <div
                                                class="progress-fill"
                                                style=move || format!("width: {}%", progress.get())
                                            />
                                            <span class="progress-label">
                                                {move || format!("{}%", progress.get())}
                                            </span>
                                        </div>
                                        
                                        <p class="progress-text">
                                            "Processing "
                                            {move || {
                                                let total = selected_ids.get().len();
                                                let current = (progress.get() as f64 / 100.0 * total as f64) as usize;
                                                format!("{} of {}", current, total)
                                            }}
                                            " records..."
                                        </p>
                                    </div>
                                }
                            >
                                <div class="workflow-list">
                                    <For
                                        each=move || workflows.get()
                                        key=|wf| wf.id
                                        children=move |workflow| {
                                            let wf = workflow.clone();
                                            view! {
                                                <div
                                                    class="workflow-item"
                                                    on:click=move |_| execute_workflow(wf.clone())
                                                >
                                                    <div class="workflow-item-header">
                                                        <h3>{workflow.name.clone()}</h3>
                                                        <svg class="icon-arrow" viewBox="0 0 24 24">
                                                            <path d="M9 5l7 7-7 7" stroke="currentColor" fill="none" />
                                                        </svg>
                                                    </div>
                                                    {workflow.description.clone().map(|desc| view! {
                                                        <p class="workflow-description">{desc}</p>
                                                    })}
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </Show>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
