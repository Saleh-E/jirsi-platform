//! Workflow Execution Panel - Real-time execution status and results
//!
//! Shows live execution status when a workflow is running,
//! with step-by-step progress and results.

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionStep {
    pub node_id: Uuid,
    pub node_label: String,
    pub status: ExecutionStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: ExecutionStatus,
    pub steps: Vec<NodeExecutionStep>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total_duration_ms: Option<u64>,
}

#[component]
pub fn ExecutionPanel(
    /// Current execution (if any)
    execution: Signal<Option<WorkflowExecution>>,
    
    /// Callback to execute workflow
    #[prop(optional)]
    on_execute: Option<Callback<Uuid>>,
    
    /// Callback to cancel execution
    #[prop(optional)]
    on_cancel: Option<Callback<Uuid>>,
) -> impl IntoView {
    view! {
        <div class="execution-panel">
            <Show
                when=move || execution.get().is_some()
                fallback=|| view! {
                    <div class="execution-empty">
                        <svg class="empty-icon" viewBox="0 0 24 24">
                            <circle cx="12" cy="12" r="10" stroke="currentColor" fill="none" />
                            <path d="M8 12l4 4 8-8" stroke="currentColor" fill="none" />
                        </svg>
                        <p>"No workflow executing"</p>
                        <p class="empty-hint">"Click 'Run' to start execution"</p>
                    </div>
                }
            >
                {move || execution.get().map(|exec| view! {
                    <div class="execution-active">
                        // Header
                        <div class="execution-header">
                            <div class="execution-title">
                                <h3>"Workflow Execution"</h3>
                                <span class="execution-id">{exec.id.to_string()[..8].to_string()}</span>
                            </div>
                            
                            <div class="execution-status">
                                {match exec.status {
                                    ExecutionStatus::Pending => view! {
                                        <span class="status-badge status-pending">
                                            "Pending"
                                        </span>
                                    },
                                    ExecutionStatus::Running => view! {
                                        <span class="status-badge status-running">
                                            <div class="spinner-small" />
                                            "Running"
                                        </span>
                                    },
                                    ExecutionStatus::Completed => view! {
                                        <span class="status-badge status-completed">
                                            "✓ Completed"
                                        </span>
                                    },
                                    ExecutionStatus::Failed => view! {
                                        <span class="status-badge status-failed">
                                            "✗ Failed"
                                        </span>
                                    },
                                }}
                            </div>
                        </div>
                        
                        // Timeline
                        <div class="execution-timeline">
                            <div class="timeline-info">
                                <span>"Started: " {exec.started_at.clone()}</span>
                                {exec.total_duration_ms.map(|ms| view! {
                                    <span>"Duration: " {format!("{}ms", ms)}</span>
                                })}
                            </div>
                        </div>
                        
                        // Steps
                        <div class="execution-steps">
                            <For
                                each=move || exec.steps.clone()
                                key=|step| step.node_id
                                children=move |step| {
                                    view! {
                                        <div
                                            class="step"
                                            class:step-pending=matches!(step.status, ExecutionStatus::Pending)
                                            class:step-running=matches!(step.status, ExecutionStatus::Running)
                                            class:step-completed=matches!(step.status, ExecutionStatus::Completed)
                                            class:step-failed=matches!(step.status, ExecutionStatus::Failed)
                                        >
                                            <div class="step-indicator">
                                                {match step.status {
                                                    ExecutionStatus::Pending => view! {
                                                        <div class="step-dot step-dot-pending" />
                                                    },
                                                    ExecutionStatus::Running => view! {
                                                        <div class="step-dot step-dot-running">
                                                            <div class="spinner-tiny" />
                                                        </div>
                                                    },
                                                    ExecutionStatus::Completed => view! {
                                                        <div class="step-dot step-dot-completed">"✓"</div>
                                                    },
                                                    ExecutionStatus::Failed => view! {
                                                        <div class="step-dot step-dot-failed">"✗"</div>
                                                    },
                                                }}
                                            </div>
                                            
                                            <div class="step-content">
                                                <div class="step-header">
                                                    <h4>{step.node_label.clone()}</h4>
                                                    {step.duration_ms.map(|ms| view! {
                                                        <span class="step-duration">{format!("{}ms", ms)}</span>
                                                    })}
                                                </div>
                                                
                                                {step.output.as_ref().map(|output| view! {
                                                    <div class="step-output">
                                                        <pre>{serde_json::to_string_pretty(output).unwrap_or_default()}</pre>
                                                    </div>
                                                })}
                                                
                                                {step.error.as_ref().map(|error| view! {
                                                    <div class="step-error">
                                                        <strong>"Error: "</strong>
                                                        {error.clone()}
                                                    </div>
                                                })}
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                        
                        // Actions
                        <div class="execution-actions">
                            {if matches!(exec.status, ExecutionStatus::Running) {
                                view! {
                                    <button
                                        class="btn btn-danger"
                                        on:click=move |_| {
                                            if let Some(callback) = on_cancel {
                                                callback.call(exec.id);
                                            }
                                        }
                                    >
                                        "Cancel Execution"
                                    </button>
                                }
                            } else if matches!(exec.status, ExecutionStatus::Failed) {
                                view! {
                                    <button
                                        class="btn btn-primary"
                                        on:click=move |_| {
                                            if let Some(callback) = on_execute {
                                                callback.call(exec.workflow_id);
                                            }
                                        }
                                    >
                                        "Retry"
                                    </button>
                                }
                            } else {
                                view! {
                                    <button
                                        class="btn btn-secondary"
                                        on:click=move |_| {
                                            // Close panel
                                        }
                                    >
                                        "Close"
                                    </button>
                                }
                            }}
                        </div>
                    </div>
                })}
            </Show>
        </div>
    }
}
