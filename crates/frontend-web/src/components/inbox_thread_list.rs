//! Inbox Thread List Component - Displays conversation threads

use leptos::*;
use crate::api::InboxThread;

/// Thread list component for inbox
#[component]
pub fn InboxThreadList(
    threads: Vec<InboxThread>,
    selected_id: Option<String>,
    on_select: impl Fn(String, String) + Clone + 'static,
) -> impl IntoView {
    if threads.is_empty() {
        return view! {
            <div class="thread-list-empty">
                <p>"No conversations yet"</p>
            </div>
        }.into_view();
    }
    
    view! {
        <div class="thread-list">
            {threads.into_iter().map(|thread| {
                let thread_id = thread.entity_id.clone();
                let entity_type = thread.entity_type.clone();
                let is_selected = selected_id.as_ref() == Some(&thread.entity_id);
                let on_select = on_select.clone();
                
                view! {
                    <ThreadItem 
                        thread=thread
                        is_selected=is_selected
                        on_click=move || {
                            on_select(thread_id.clone(), entity_type.clone());
                        }
                    />
                }
            }).collect_view()}
        </div>
    }.into_view()
}

/// Individual thread item component
#[component]
fn ThreadItem(
    thread: InboxThread,
    is_selected: bool,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let class = if is_selected {
        "thread-item selected"
    } else {
        "thread-item"
    };
    
    // Get avatar initial
    let avatar_initial = thread.entity_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    
    // Format relative time
    let relative_time = format_relative_time(&thread.last_message_at);
    
    // Determine icon for interaction type
    let type_icon = match thread.last_interaction_type.to_lowercase().as_str() {
        "email" => "✉️",
        "note" => "📝",
        "call" => "📞",
        "message" | "whatsapp" => "💬",
        "meeting" => "📅",
        _ => "📄",
    };
    
    view! {
        <div class=class on:click=move |_| on_click()>
            <div class="thread-avatar">
                <span class="avatar-initial">{avatar_initial}</span>
            </div>
            <div class="thread-content">
                <div class="thread-header">
                    <span class="thread-name">{thread.entity_name}</span>
                    <span class="thread-time">{relative_time}</span>
                </div>
                <div class="thread-preview">
                    <span class="thread-type-icon">{type_icon}</span>
                    <span class="thread-message">{thread.last_message_preview}</span>
                </div>
            </div>
            {if thread.unread_count > 0 {
                view! {
                    <div class="thread-unread-badge">
                        <span>{thread.unread_count}</span>
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

/// Format timestamp to relative time (e.g., "5m ago")
fn format_relative_time(timestamp: &str) -> String {
    use chrono::{DateTime, Utc};
    
    let parsed: Result<DateTime<Utc>, _> = timestamp.parse();
    let now = Utc::now();
    
    match parsed {
        Ok(dt) => {
            let duration = now.signed_duration_since(dt);
            
            if duration.num_seconds() < 60 {
                "Just now".to_string()
            } else if duration.num_minutes() < 60 {
                format!("{}m ago", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                format!("{}h ago", duration.num_hours())
            } else if duration.num_days() < 7 {
                format!("{}d ago", duration.num_days())
            } else {
                format!("{}w ago", duration.num_weeks())
            }
        }
        Err(_) => timestamp.to_string(),
    }
}
