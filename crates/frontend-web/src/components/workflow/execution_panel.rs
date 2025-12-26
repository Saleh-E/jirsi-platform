//! Workflow Execution Panel - Real-time execution status and results
//!
//! Shows live execution status when a workflow is running,
//! with step-by-step progress, node highlighting, and WebSocket updates.

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    pub node_type: String,
    pub status: ExecutionStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub started_at: Option<String>,
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
    pub current_node_idx: Option<usize>,
}

impl WorkflowExecution {
    /// Calculate the progress percentage (0-100)
    pub fn progress_percent(&self) -> u8 {
        if self.steps.is_empty() {
            return 0;
        }
        let completed = self.steps.iter()
            .filter(|s| matches!(s.status, ExecutionStatus::Completed | ExecutionStatus::Failed))
            .count();
        ((completed * 100) / self.steps.len()) as u8
    }
}

/// Execution Panel Component - displays workflow execution progress
#[component]
pub fn ExecutionPanel(
    /// Current workflow execution (None = no execution running)
    #[prop(into)] execution: Signal<Option<WorkflowExecution>>,
    /// Callback when user wants to highlight a specific node in the canvas
    #[prop(optional)] on_highlight_node: Option<Callback<Uuid>>,
    /// Callback when user wants to cancel execution
    #[prop(optional)] on_cancel: Option<Callback<Uuid>>,
    /// Callback when user wants to retry execution
    #[prop(optional)] on_retry: Option<Callback<Uuid>>,
) -> impl IntoView {
    view! {
        <div class="execution-panel bg-gray-900 border border-gray-700 rounded-lg overflow-hidden">
            <div class="panel-header flex items-center justify-between px-4 py-3 bg-gray-800 border-b border-gray-700">
                <div class="flex items-center gap-2">
                    <i class="fa-solid fa-play-circle text-brand-400"></i>
                    <span class="font-medium text-white">"Execution"</span>
                </div>
            </div>
            
            <div class="panel-content p-4">
                {move || {
                    match execution.get() {
                        None => view! {
                            <div class="empty-state text-center py-8">
                                <i class="fa-solid fa-play-circle text-4xl text-gray-600 mb-3"></i>
                                <p class="text-gray-400">"No execution running"</p>
                                <p class="text-gray-500 text-sm">"Click 'Run' to start"</p>
                            </div>
                        }.into_view(),
                        Some(exec) => {
                            let exec_id = exec.id;
                            let exec_status = exec.status;
                            let exec_id_short = exec.id.to_string()[..8].to_string();
                            let progress = exec.progress_percent();
                            let total_steps = exec.steps.len();
                            let completed_steps = exec.steps.iter()
                                .filter(|s| matches!(s.status, ExecutionStatus::Completed))
                                .count();
                            let steps = exec.steps.clone();
                            let duration = exec.total_duration_ms;
                            
                            view! {
                                <div class="execution-active">
                                    // Status Header
                                    <div class="flex items-center gap-3 mb-4">
                                        <div class=format!(
                                            "w-10 h-10 rounded-full flex items-center justify-center {}",
                                            match exec_status {
                                                ExecutionStatus::Running => "bg-brand-500/20 animate-pulse",
                                                ExecutionStatus::Completed => "bg-success-500/20",
                                                ExecutionStatus::Failed => "bg-danger-500/20",
                                                ExecutionStatus::Pending => "bg-gray-500/20",
                                            }
                                        )>
                                            <i class=format!(
                                                "fa-solid {}",
                                                match exec_status {
                                                    ExecutionStatus::Running => "fa-cog fa-spin text-brand-400",
                                                    ExecutionStatus::Completed => "fa-check text-success-500",
                                                    ExecutionStatus::Failed => "fa-times text-danger-500",
                                                    ExecutionStatus::Pending => "fa-clock text-gray-400",
                                                }
                                            )></i>
                                        </div>
                                        <div>
                                            <div class="font-medium text-white">
                                                {match exec_status {
                                                    ExecutionStatus::Running => "Executing...",
                                                    ExecutionStatus::Completed => "Completed",
                                                    ExecutionStatus::Failed => "Failed",
                                                    ExecutionStatus::Pending => "Pending",
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                {"ID: "} {exec_id_short}
                                            </div>
                                        </div>
                                        {duration.map(|ms| view! {
                                            <div class="ml-auto text-sm text-gray-400">
                                                <i class="fa-solid fa-clock mr-1"></i>
                                                {if ms >= 1000 {
                                                    format!("{:.1}s", ms as f64 / 1000.0)
                                                } else {
                                                    format!("{}ms", ms)
                                                }}
                                            </div>
                                        })}
                                    </div>
                                    
                                    // Progress Bar
                                    <Show when=move || exec_status == ExecutionStatus::Running>
                                        <div class="mb-4">
                                            <div class="h-2 bg-gray-700 rounded-full overflow-hidden">
                                                <div 
                                                    class="h-full bg-gradient-to-r from-brand-500 to-brand-400 transition-all duration-300"
                                                    style=format!("width: {}%", progress)
                                                ></div>
                                            </div>
                                            <div class="flex justify-between text-xs text-gray-400 mt-1">
                                                <span>{format!("{}/{} nodes", completed_steps, total_steps)}</span>
                                                <span>{format!("{}%", progress)}</span>
                                            </div>
                                        </div>
                                    </Show>
                                    
                                    // Steps List
                                    <div class="steps-list space-y-2 max-h-60 overflow-y-auto">
                                        {steps.into_iter().enumerate().map(|(idx, step)| {
                                            let node_id = step.node_id;
                                            view! {
                                                <div 
                                                    class="step flex items-center gap-3 p-3 bg-gray-800 rounded-lg hover:bg-gray-750 cursor-pointer"
                                                    on:click=move |_| {
                                                        if let Some(cb) = on_highlight_node {
                                                            cb.call(node_id);
                                                        }
                                                    }
                                                >
                                                    <div class=format!(
                                                        "w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium {}",
                                                        match step.status {
                                                            ExecutionStatus::Pending => "bg-gray-600 text-gray-300",
                                                            ExecutionStatus::Running => "bg-brand-500 text-white",
                                                            ExecutionStatus::Completed => "bg-success-500 text-white",
                                                            ExecutionStatus::Failed => "bg-danger-500 text-white",
                                                        }
                                                    )>
                                                        {match step.status {
                                                            ExecutionStatus::Pending => view! { <span>{idx + 1}</span> }.into_view(),
                                                            ExecutionStatus::Running => view! { <i class="fa-solid fa-spinner fa-spin"></i> }.into_view(),
                                                            ExecutionStatus::Completed => view! { <i class="fa-solid fa-check"></i> }.into_view(),
                                                            ExecutionStatus::Failed => view! { <i class="fa-solid fa-times"></i> }.into_view(),
                                                        }}
                                                    </div>
                                                    <div class="flex-1 min-w-0">
                                                        <div class="font-medium text-white truncate">{step.node_label}</div>
                                                        <div class="text-xs text-gray-400">{step.node_type}</div>
                                                    </div>
                                                    {step.duration_ms.map(|ms| view! {
                                                        <div class="text-xs text-gray-500">{format!("{}ms", ms)}</div>
                                                    })}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                    
                                    // Action Buttons
                                    <div class="flex gap-2 mt-4">
                                        <Show when=move || exec_status == ExecutionStatus::Running>
                                            <button 
                                                class="flex-1 px-3 py-2 bg-danger-500/20 text-danger-400 rounded-lg hover:bg-danger-500/30"
                                                on:click=move |_| {
                                                    if let Some(cb) = on_cancel {
                                                        cb.call(exec_id);
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-stop mr-2"></i>
                                                "Cancel"
                                            </button>
                                        </Show>
                                        <Show when=move || exec_status == ExecutionStatus::Failed>
                                            <button 
                                                class="flex-1 px-3 py-2 bg-brand-500/20 text-brand-400 rounded-lg hover:bg-brand-500/30"
                                                on:click=move |_| {
                                                    if let Some(cb) = on_retry {
                                                        cb.call(exec_id);
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-redo mr-2"></i>
                                                "Retry"
                                            </button>
                                        </Show>
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }
                }}
            </div>
        </div>
    }
}
