//! Toast Notifications Component
//!
//! Provides global toast notifications with programmatic access.
//! Supports: success, error, warning, info levels.

use leptos::*;
use crate::context::socket::{WsEvent, SocketState};

/// Toast notification item
#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u32,
    pub title: String,
    pub message: String,
    pub level: String, // info, success, warning, error
}

/// Global toast context - provides `show_toast` function anywhere in the app
#[derive(Clone)]
pub struct ToastContext {
    pub show: Callback<Toast>,
}

/// Hook to get the toast context and show toasts
pub fn use_toast() -> Option<ToastContext> {
    use_context::<ToastContext>()
}

/// Helper to show a success toast
pub fn show_success(ctx: &ToastContext, message: &str) {
    ctx.show.call(Toast {
        id: 0, // Will be assigned by provider
        title: "Success".to_string(),
        message: message.to_string(),
        level: "success".to_string(),
    });
}

/// Helper to show an error toast
pub fn show_error(ctx: &ToastContext, message: &str) {
    ctx.show.call(Toast {
        id: 0,
        title: "Error".to_string(),
        message: message.to_string(),
        level: "error".to_string(),
    });
}

/// Helper to show a warning toast
pub fn show_warning(ctx: &ToastContext, message: &str) {
    ctx.show.call(Toast {
        id: 0,
        title: "Warning".to_string(),
        message: message.to_string(),
        level: "warning".to_string(),
    });
}

/// Helper to show an info toast
pub fn show_info(ctx: &ToastContext, message: &str) {
    ctx.show.call(Toast {
        id: 0,
        title: "Info".to_string(),
        message: message.to_string(),
        level: "info".to_string(),
    });
}

/// Toast Provider - wrap your app with this to enable global toasts
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = create_signal::<Vec<Toast>>(Vec::new());
    let (next_id, set_next_id) = create_signal(0u32);
    
    // Create callback for adding toasts
    let add_toast = Callback::new(move |mut toast: Toast| {
        let id = next_id.get();
        toast.id = id;
        set_next_id.update(|n| *n += 1);
        set_toasts.update(|v| v.push(toast));
        
        // Auto-dismiss after 5 seconds
        set_timeout(
            move || {
                set_toasts.update(|v| {
                    v.retain(|t| t.id != id);
                });
            },
            std::time::Duration::from_secs(5),
        );
    });
    
    // Provide context
    provide_context(ToastContext { show: add_toast.clone() });
    
    // Also integrate with WebSocket events
    let socket_ctx = use_context::<crate::context::socket::SocketContext>();
    if let Some(ctx) = socket_ctx {
        let add_toast_ws = add_toast.clone();
        create_effect(move |prev_count: Option<usize>| {
            let events = ctx.events.get();
            let current_count = events.len();
            
            if let Some(prev) = prev_count {
                if current_count > prev {
                    for event in events.iter().skip(prev) {
                        let toast = match event {
                            WsEvent::NewMessage { sender_name, preview, .. } => Some(Toast {
                                id: 0,
                                title: format!("New message from {}", sender_name),
                                message: preview.chars().take(50).collect(),
                                level: "info".to_string(),
                            }),
                            WsEvent::LeadAssigned { contact_name, assigned_by, .. } => Some(Toast {
                                id: 0,
                                title: "Lead Assigned".to_string(),
                                message: format!("{} assigned by {}", contact_name, assigned_by),
                                level: "success".to_string(),
                            }),
                            WsEvent::Notification { title, message, level } => Some(Toast {
                                id: 0,
                                title: title.clone(),
                                message: message.clone(),
                                level: level.clone(),
                            }),
                            WsEvent::Connected { .. } => Some(Toast {
                                id: 0,
                                title: "Connected".to_string(),
                                message: "Real-time updates enabled".to_string(),
                                level: "success".to_string(),
                            }),
                            _ => None,
                        };
                        
                        if let Some(t) = toast {
                            add_toast_ws.call(t);
                        }
                    }
                }
            }
            
            current_count
        });
    }
    
    view! {
        {children()}
        <ToastContainer toasts=toasts set_toasts=set_toasts />
    }
}

/// Toast container - renders the actual toast notifications
#[component]
fn ToastContainer(
    toasts: ReadSignal<Vec<Toast>>,
    set_toasts: WriteSignal<Vec<Toast>>,
) -> impl IntoView {
    view! {
        <div class="toast-container">
            <For
                each=move || toasts.get().into_iter().enumerate()
                key=|(_, t)| t.id
                children=move |(_, toast)| {
                    let level_class = format!("toast toast-{}", toast.level);
                    let toast_id = toast.id;
                    
                    // Icon based on level
                    let icon = match toast.level.as_str() {
                        "success" => "✓",
                        "error" => "✕",
                        "warning" => "!",
                        _ => "i",
                    };
                    
                    view! {
                        <div class=level_class>
                            <div class="toast-icon">{icon}</div>
                            <div class="toast-content">
                                <div class="toast-title">{toast.title.clone()}</div>
                                <div class="toast-message">{toast.message.clone()}</div>
                            </div>
                            <button
                                class="toast-close"
                                on:click=move |_| {
                                    set_toasts.update(|v| {
                                        v.retain(|t| t.id != toast_id);
                                    });
                                }
                            >
                                "×"
                            </button>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// Connection status indicator
#[component]
pub fn ConnectionStatus() -> impl IntoView {
    let socket_ctx = use_context::<crate::context::socket::SocketContext>();

    view! {
        {move || {
            if let Some(ctx) = &socket_ctx {
                let state = ctx.state.get();
                let (class, text) = match state {
                    SocketState::Connected => ("status-connected", "●"),
                    SocketState::Connecting => ("status-connecting", "○"),
                    SocketState::Disconnected => ("status-disconnected", "○"),
                    SocketState::Error(_) => ("status-error", "!"),
                };
                view! {
                    <span class=format!("connection-status {}", class) title=format!("{:?}", state)>
                        {text}
                    </span>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }
        }}
    }
}
