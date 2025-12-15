//! Composer Component - Message input with type selector and keyboard shortcuts

use leptos::*;

/// Message composer with textarea, type selector, and send button
#[component]
pub fn Composer(
    content: ReadSignal<String>,
    set_content: WriteSignal<String>,
    message_type: ReadSignal<String>,
    set_message_type: WriteSignal<String>,
    on_send: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let on_send_clone = on_send.clone();
    
    // Handle Ctrl+Enter to send
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && ev.ctrl_key() {
            ev.prevent_default();
            on_send_clone();
        }
    };
    
    view! {
        <div class="composer">
            // Type selector
            <div class="composer-type-selector">
                <button 
                    class=move || if message_type.get() == "email" { "type-btn active" } else { "type-btn" }
                    on:click=move |_| set_message_type.set("email".to_string())
                    title="Send as Email"
                >
                    "✉️ Email"
                </button>
                <button 
                    class=move || if message_type.get() == "note" { "type-btn active" } else { "type-btn" }
                    on:click=move |_| set_message_type.set("note".to_string())
                    title="Add Internal Note"
                >
                    "📝 Note"
                </button>
                <button 
                    class=move || if message_type.get() == "whatsapp" { "type-btn active" } else { "type-btn" }
                    on:click=move |_| set_message_type.set("whatsapp".to_string())
                    title="Send via WhatsApp"
                >
                    "💬 WhatsApp"
                </button>
            </div>
            
            // Text input area
            <div class="composer-input-area">
                <textarea 
                    class="composer-textarea"
                    placeholder="Type your message... (Ctrl+Enter to send)"
                    prop:value=move || content.get()
                    on:input=move |ev| {
                        set_content.set(event_target_value(&ev));
                    }
                    on:keydown=on_keydown
                />
            </div>
            
            // Actions row
            <div class="composer-actions">
                <button class="attach-btn" title="Attach file">
                    "📎"
                </button>
                <button 
                    class="send-btn"
                    on:click=move |_| on_send()
                    disabled=move || content.get().trim().is_empty()
                >
                    "Send " {move || match message_type.get().as_str() {
                        "note" => "Note",
                        "whatsapp" => "WhatsApp",
                        _ => "Email",
                    }}
                </button>
            </div>
        </div>
    }
}
