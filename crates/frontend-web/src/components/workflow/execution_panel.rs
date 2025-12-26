//! Workflow Execution Panel - Real-time execution status and results
//!
//! Shows live execution status when a workflow is running,
//! with step-by-step progress, node highlighting, and WebSocket updates.

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
        ((completed as f32 / self.steps.len() as f32) * 100.0) as u8
    }

    /// Get the currently executing node
    pub fn current_node(&self) -> Option<&NodeExecutionStep> {
        self.steps.iter().find(|s| matches!(s.status, ExecutionStatus::Running))
    }
}

#[component]
pub fn ExecutionPanel(
    /// Current execution (if any)
    execution: Signal<Option<WorkflowExecution>>,
    
    /// Callback to highlight a node on the canvas
    #[prop(optional)]
    on_highlight_node: Option<Callback<Uuid>>,
    
    /// Callback to execute workflow
    #[prop(optional)]
    on_execute: Option<Callback<Uuid>>,
    
    /// Callback to cancel execution
    #[prop(optional)]
    on_cancel: Option<Callback<Uuid>>,
) -> impl IntoView {
    // Track if panel is expanded
    let (expanded, set_expanded) = create_signal(true);
    
    // Auto-scroll to current step
    let steps_container_ref = create_node_ref::<leptos::html::Div>();
    
    create_effect(move |_| {
        if let Some(exec) = execution.get() {
            if matches!(exec.status, ExecutionStatus::Running) {
                // Auto-scroll to current step
                if let Some(container) = steps_container_ref.get() {
                    let current_idx = exec.current_node_idx.unwrap_or(0);
                    let scroll_top = current_idx as f64 * 80.0; // ~80px per step
                    container.set_scroll_top(scroll_top as i32);
                }
            }
        }
    });
    
    view! {
        <div class="execution-panel" class:collapsed=move || !expanded.get()>
            // Collapse/Expand header
            <div 
                class="execution-panel-header cursor-pointer" 
                on:click=move |_| set_expanded.update(|e| *e = !*e)
            >
                <div class="flex items-center gap-2">
                    <i class=move || {
                        if expanded.get() { "fa-solid fa-chevron-down" } 
                        else { "fa-solid fa-chevron-right" }
                    }></i>
                    <span>"Execution"</span>
                </div>
                
                // Mini status when collapsed
                <Show when=move || !expanded.get() && execution.get().is_some()>
                    {move || execution.get().map(|exec| view! {
                        <div class="mini-status">
                            {match exec.status {
                                ExecutionStatus::Running => view! {
                                    <span class="text-brand-400">
                                        <i class="fa-solid fa-spinner fa-spin mr-1"></i>
                                        {format!("{}%", exec.progress_percent())}
                                    </span>
                                },
                                ExecutionStatus::Completed => view! {
                                    <span class="text-success-500">"✓ Done"</span>
                                },
                                ExecutionStatus::Failed => view! {
                                    <span class="text-danger-500">"✗ Failed"</span>
                                },
                                ExecutionStatus::Pending => view! {
                                    <span class="text-gray-400">"Pending"</span>
                                },
                            }}
                        </div>
                    })}
                </Show>
            </div>
            
            <Show when=move || expanded.get()>
                <div class="execution-panel-content">
                    <Show
                        when=move || execution.get().is_some()
                        fallback=|| view! {
                            <div class="execution-empty text-center py-8">
                                <i class="fa-solid fa-play-circle text-4xl text-gray-500 mb-4"></i>
                                <p class="text-gray-400">"No workflow executing"</p>
                                <p class="text-gray-500 text-sm">"Click 'Run' to start"</p>
                            </div>
                        }
                    >
                        {move || execution.get().map(|exec| view! {
                            <div class="execution-active">
                                // Status Header
                                <div class="execution-header flex justify-between items-center p-4 border-b border-gray-700">
                                    <div class="flex items-center gap-3">
                                        <div class="execution-icon">
                                            {match exec.status {
                                                ExecutionStatus::Running => view! {
                                                    <div class="w-10 h-10 rounded-full bg-brand-500/20 flex items-center justify-center animate-pulse-soft">
                                                        <i class="fa-solid fa-cog fa-spin text-brand-400"></i>
                                                    </div>
                                                },
                                                ExecutionStatus::Completed => view! {
                                                    <div class="w-10 h-10 rounded-full bg-success-500/20 flex items-center justify-center">
                                                        <i class="fa-solid fa-check text-success-500"></i>
                                                    </div>
                                                },
                                                ExecutionStatus::Failed => view! {
                                                    <div class="w-10 h-10 rounded-full bg-danger-500/20 flex items-center justify-center">
                                                        <i class="fa-solid fa-times text-danger-500"></i>
                                                    </div>
                                                },
                                                ExecutionStatus::Pending => view! {
                                                    <div class="w-10 h-10 rounded-full bg-gray-500/20 flex items-center justify-center">
                                                        <i class="fa-solid fa-clock text-gray-400"></i>
                                                    </div>
                                                },
                                            }}
                                        </div>
                                        <div>
                                            <div class="font-medium text-white">
                                                {match exec.status {
                                                    ExecutionStatus::Running => "Executing...",
                                                    ExecutionStatus::Completed => "Completed",
                                                    ExecutionStatus::Failed => "Failed",
                                                    ExecutionStatus::Pending => "Pending",
                                                }}
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                {"ID: "} {exec.id.to_string()[..8].to_string()}
                                            </div>
                                        </div>
                                    </div>
                                    
                                    {exec.total_duration_ms.map(|ms| view! {
                                        <div class="text-sm text-gray-400">
                                            <i class="fa-solid fa-clock mr-1"></i>
                                            {if ms >= 1000 {
                                                format!("{:.1}s", ms as f64 / 1000.0)
                                            } else {
                                                format!("{}ms", ms)
                                            }}
                                        </div>
                                    })}
                                </div>
                                
                                // Progress Bar (for running executions)
                                <Show when=move || matches!(exec.status, ExecutionStatus::Running)>
                                    <div class="px-4 py-2">
                                        <div class="h-2 bg-gray-700 rounded-full overflow-hidden">
                                            <div 
                                                class="h-full bg-gradient-to-r from-brand-500 to-brand-400 transition-all duration-300 animate-shimmer"
                                                style={format!("width: {}%", exec.progress_percent())}
                                            ></div>
                                        </div>
                                        <div class="flex justify-between text-xs text-gray-400 mt-1">
                                            <span>{format!("{}/{} nodes", 
                                                exec.steps.iter().filter(|s| matches!(s.status, ExecutionStatus::Completed)).count(),
                                                exec.steps.len()
                                            )}</span>
                                            <span>{format!("{}%", exec.progress_percent())}</span>
                                        </div>
                                    </div>
                                </Show>
                                
                                // Steps List
                                <div 
                                    class="execution-steps max-h-80 overflow-y-auto" 
                                    node_ref=steps_container_ref
                                >
                                    <For
                                        each=move || exec.steps.clone().into_iter().enumerate()
                                        key=|(_, step)| step.node_id
                                        children=move |(idx, step)| {
                                            let node_id = step.node_id;
                                            let is_current = matches!(step.status, ExecutionStatus::Running);
                                            
                                            view! {
                                                <div
                                                    class="step flex gap-4 p-4 border-b border-gray-800 hover:bg-gray-800/50 cursor-pointer transition-colors"
                                                    class:bg-brand-500/10=is_current
                                                    class:border-l-4=is_current
                                                    class:border-l-brand-500=is_current
                                                    on:click=move |_| {
                                                        if let Some(callback) = on_highlight_node {
                                                            callback.call(node_id);
                                                        }
                                                    }
                                                >
                                                    // Step Number + Status Icon
                                                    <div class="flex-shrink-0 flex flex-col items-center">
                                                        <div class=move || format!(
                                                            "w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium {}",
                                                            match step.status {
                                                                ExecutionStatus::Pending => "bg-gray-700 text-gray-400",
                                                                ExecutionStatus::Running => "bg-brand-500 text-white animate-pulse-soft",
                                                                ExecutionStatus::Completed => "bg-success-500 text-white",
                                                                ExecutionStatus::Failed => "bg-danger-500 text-white",
                                                            }
                                                        )>
                                                            {match step.status {
                                                                ExecutionStatus::Pending => view! { <span>{idx + 1}</span> },
                                                                ExecutionStatus::Running => view! { <i class="fa-solid fa-spinner fa-spin"></i> },
                                                                ExecutionStatus::Completed => view! { <i class="fa-solid fa-check"></i> },
                                                                ExecutionStatus::Failed => view! { <i class="fa-solid fa-times"></i> },
                                                            }}
                                                        </div>
                                                        
                                                        // Connector line
                                                        <Show when=move || idx < exec.steps.len() - 1>
                                                            <div class="w-px h-8 bg-gray-700 mt-1"></div>
                                                        </Show>
                                                    </div>
                                                    
                                                    // Step Content
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-center justify-between mb-1">
                                                            <div class="font-medium text-white truncate">
                                                                {step.node_label.clone()}
                                                            </div>
                                                            {step.duration_ms.map(|ms| view! {
                                                                <span class="text-xs text-gray-400">{format!("{}ms", ms)}</span>
                                                            })}
                                                        </div>
                                                        
                                                        <div class="text-xs text-gray-500 mb-2">
                                                            {step.node_type.clone()}
                                                        </div>
                                                        
                                                        // Output Preview
                                                        {step.output.as_ref().map(|output| view! {
                                                            <div class="bg-gray-900 rounded p-2 text-xs font-mono text-gray-300 max-h-20 overflow-y-auto">
                                                                <pre class="whitespace-pre-wrap">
                                                                    {serde_json::to_string_pretty(output)
                                                                        .unwrap_or_default()
                                                                        .chars()
                                                                        .take(200)
                                                                        .collect::<String>()
                                                                    }
                                                                    {if serde_json::to_string(output).unwrap_or_default().len() > 200 { "..." } else { "" }}
                                                                </pre>
                                                            </div>
                                                        })}
                                                        
                                                        // Error Display
                                                        {step.error.as_ref().map(|error| view! {
                                                            <div class="bg-danger-500/10 border border-danger-500/30 rounded p-2 mt-2">
                                                                <div class="flex items-center gap-2 text-danger-400 text-sm">
                                                                    <i class="fa-solid fa-exclamation-triangle"></i>
                                                                    <span class="font-medium">"Error"</span>
                                                                </div>
                                                                <p class="text-xs text-gray-300 mt-1">{error.clone()}</p>
                                                            </div>
                                                        })}
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                                
                                // Actions Footer
                                <div class="execution-actions p-4 border-t border-gray-700 flex gap-2">
                                    {if matches!(exec.status, ExecutionStatus::Running) {
                                        view! {
                                            <button
                                                class="btn btn-danger flex-1"
                                                on:click=move |_| {
                                                    if let Some(callback) = on_cancel {
                                                        callback.call(exec.id);
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-stop mr-2"></i>
                                                "Cancel"
                                            </button>
                                        }
                                    } else if matches!(exec.status, ExecutionStatus::Failed) {
                                        view! {
                                            <button
                                                class="btn btn-primary flex-1"
                                                on:click=move |_| {
                                                    if let Some(callback) = on_execute {
                                                        callback.call(exec.workflow_id);
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-redo mr-2"></i>
                                                "Retry"
                                            </button>
                                        }
                                    } else {
                                        view! {
                                            <button
                                                class="btn btn-secondary flex-1"
                                                on:click=move |_| {
                                                    if let Some(callback) = on_execute {
                                                        callback.call(exec.workflow_id);
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-play mr-2"></i>
                                                "Run Again"
                                            </button>
                                        }
                                    }}
                                </div>
                            </div>
                        })}
                    </Show>
                </div>
            </Show>
        </div>
    }
}

