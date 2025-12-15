//! Toast Notifications Component
//!
//! Shows real-time toast notifications for WebSocket events.

use leptos::*;
use crate::context::socket::{use_socket, WsEvent, SocketState};

/// Toast notification item
#[derive(Clone)]
struct Toast {
    id: u32,
    title: String,
    message: String,
    level: String, // info, success, warning, error
}

/// Toast container component - shows notifications from WebSocket events
#[component]
pub fn ToastContainer() -> impl IntoView {
    let (toasts, set_toasts) = create_signal::<Vec<Toast>>(Vec::new());
    let (next_id, set_next_id) = create_signal(0u32);

    // Try to get socket context (may not exist if not logged in)
    let socket_ctx = use_context::<crate::context::socket::SocketContext>();

    // Listen for new events
    if let Some(ctx) = socket_ctx {
        create_effect(move |prev_count: Option<usize>| {
            let events = ctx.events.get();
            let current_count = events.len();
            
            // Only process if we have new events
            if let Some(prev) = prev_count {
                if current_count > prev {
                    // Process new events
                    for event in events.iter().skip(prev) {
                        let toast = match event {
                            WsEvent::NewMessage { sender_name, preview, .. } => Some(Toast {
                                id: next_id.get(),
                                title: format!("New message from {}", sender_name),
                                message: preview.chars().take(50).collect(),
                                level: "info".to_string(),
                            }),
                            WsEvent::LeadAssigned { contact_name, assigned_by, .. } => Some(Toast {
                                id: next_id.get(),
                                title: "Lead Assigned".to_string(),
                                message: format!("{} assigned by {}", contact_name, assigned_by),
                                level: "success".to_string(),
                            }),
                            WsEvent::Notification { title, message, level } => Some(Toast {
                                id: next_id.get(),
                                title: title.clone(),
                                message: message.clone(),
                                level: level.clone(),
                            }),
                            WsEvent::Connected { .. } => Some(Toast {
                                id: next_id.get(),
                                title: "Connected".to_string(),
                                message: "Real-time updates enabled".to_string(),
                                level: "success".to_string(),
                            }),
                            _ => None,
                        };

                        if let Some(t) = toast {
                            let toast_id = t.id;
                            set_next_id.update(|id| *id += 1);
                            set_toasts.update(|v| v.push(t));

                            // Auto-dismiss after 5 seconds
                            set_timeout(
                                move || {
                                    set_toasts.update(|v| {
                                        v.retain(|t| t.id != toast_id);
                                    });
                                },
                                std::time::Duration::from_secs(5),
                            );
                        }
                    }
                }
            }
            
            current_count
        });
    }

    view! {
        <div class="toast-container">
            <For
                each=move || toasts.get().into_iter().enumerate()
                key=|(_, t)| t.id
                children=move |(_, toast)| {
                    let level_class = format!("toast toast-{}", toast.level);
                    view! {
                        <div class=level_class>
                            <div class="toast-header">
                                <strong>{toast.title.clone()}</strong>
                                <button
                                    class="toast-close"
                                    on:click={
                                        let id = toast.id;
                                        move |_| {
                                            set_toasts.update(|v| {
                                                v.retain(|t| t.id != id);
                                            });
                                        }
                                    }
                                >
                                    "×"
                                </button>
                            </div>
                            <div class="toast-body">
                                {toast.message.clone()}
                            </div>
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
