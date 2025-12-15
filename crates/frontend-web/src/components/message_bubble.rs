//! Message Bubble Component - Displays individual messages with type styling

use leptos::*;
use crate::api::ThreadMessage;

/// Message bubble component with direction and type-specific styling
#[component]
pub fn MessageBubble(message: ThreadMessage) -> impl IntoView {
    // Determine CSS classes based on direction and type
    let direction_class = if message.direction == "inbound" {
        "inbound"
    } else {
        "outbound"
    };
    
    // Type-specific class for styling
    let type_class = match message.interaction_type.to_lowercase().as_str() {
        "note" => "note",               // Yellow/amber for internal notes
        "email" => "email",             // White background
        "whatsapp" | "message" => "message",  // Blue/green background
        "call" => "call",
        _ => "default",
    };
    
    let bubble_class = format!("message-bubble {} {}", direction_class, type_class);
    
    // Format timestamp
    let time_display = format_message_time(&message.occurred_at);
    
    // Type icon
    let type_icon = match message.interaction_type.to_lowercase().as_str() {
        "email" => "✉️",
        "note" => "📝",
        "call" => "📞",
        "message" | "whatsapp" => "💬",
        "meeting" => "📅",
        _ => "",
    };
    
    view! {
        <div class=bubble_class>
            <div class="bubble-header">
                <span class="bubble-type-icon">{type_icon}</span>
                <span class="bubble-title">{message.title.clone()}</span>
                <span class="bubble-time">{time_display}</span>
            </div>
            {message.content.as_ref().map(|content| {
                view! {
                    <div class="bubble-content">
                        {content.clone()}
                    </div>
                }
            })}
            {message.duration_minutes.map(|mins| {
                view! {
                    <div class="bubble-meta">
                        <span class="call-duration">"Duration: " {mins} " mins"</span>
                    </div>
                }
            })}
        </div>
    }
}

/// Format message timestamp for display
fn format_message_time(timestamp: &str) -> String {
    use chrono::{DateTime, Utc, Local};
    
    let parsed: Result<DateTime<Utc>, _> = timestamp.parse();
    
    match parsed {
        Ok(dt) => {
            let local: DateTime<Local> = dt.into();
            local.format("%b %d, %H:%M").to_string()
        }
        Err(_) => timestamp.to_string(),
    }
}
