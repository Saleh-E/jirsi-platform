//! Inbox Page - Unified 3-pane communication inbox

use leptos::*;
use crate::api::{fetch_inbox_threads, fetch_thread_messages, send_inbox_reply};
use crate::components::inbox_thread_list::InboxThreadList;
use crate::components::message_bubble::MessageBubble;
use crate::components::composer::Composer;

/// Main Inbox Page with 3-pane layout
#[component]
pub fn InboxPage() -> impl IntoView {
    // Filter state
    let (active_filter, set_active_filter) = create_signal("all".to_string());
    
    // Thread list resource
    let threads_resource = create_local_resource(
        move || active_filter.get(),
        |filter| async move {
            fetch_inbox_threads(&filter).await.unwrap_or_default()
        }
    );
    
    // Selected thread state
    let (selected_thread_id, set_selected_thread_id) = create_signal::<Option<String>>(None);
    let (selected_entity_type, set_selected_entity_type) = create_signal::<Option<String>>(None);
    
    // Messages for selected thread
    let messages_resource = create_local_resource(
        move || selected_thread_id.get(),
        |thread_id| async move {
            match thread_id {
                Some(id) => fetch_thread_messages(&id).await.ok(),
                None => None,
            }
        }
    );
    
    // Reply handler
    let (reply_content, set_reply_content) = create_signal(String::new());
    let (reply_type, set_reply_type) = create_signal("email".to_string());
    
    let send_action = create_action(move |input: &(String, String, String)| {
        let (entity_id, content, msg_type) = input.clone();
        async move {
            let _ = send_inbox_reply(&entity_id, &content, &msg_type).await;
            set_reply_content.set(String::new());
        }
    });
    
    // Auto-refresh threads every 30 seconds
    #[cfg(target_arch = "wasm32")]
    {
        let _ = set_interval_with_handle(
            move || {
                threads_resource.refetch();
            },
            std::time::Duration::from_secs(30),
        );
    }
    
    // Handle thread selection
    let on_select_thread = move |entity_id: String, entity_type: String| {
        set_selected_thread_id.set(Some(entity_id));
        set_selected_entity_type.set(Some(entity_type));
    };
    
    // Handle back navigation (for mobile)
    let on_back = move |_| {
        set_selected_thread_id.set(None);
        set_selected_entity_type.set(None);
    };

    view! {
        <div class="inbox-page">
            // Mobile Filter Tabs (visible only on mobile via CSS)
            <div class="inbox-mobile-tabs">
                <button 
                    class=move || if active_filter.get() == "all" { "mobile-tab active" } else { "mobile-tab" }
                    on:click=move |_| set_active_filter.set("all".to_string())
                >
                    "All"
                </button>
                <button 
                    class=move || if active_filter.get() == "unread" { "mobile-tab active" } else { "mobile-tab" }
                    on:click=move |_| set_active_filter.set("unread".to_string())
                >
                    "Unread"
                </button>
                <button 
                    class=move || if active_filter.get() == "assigned" { "mobile-tab active" } else { "mobile-tab" }
                    on:click=move |_| set_active_filter.set("assigned".to_string())
                >
                    "Assigned"
                </button>
                <button 
                    class=move || if active_filter.get() == "sent" { "mobile-tab active" } else { "mobile-tab" }
                    on:click=move |_| set_active_filter.set("sent".to_string())
                >
                    "Sent"
                </button>
            </div>
            
            <div class="inbox-layout">
                // Pane 1: Filters (desktop only)
                <aside class="inbox-filters">
                    <h2 class="filters-title">"Inbox"</h2>
                    <nav class="filter-nav">
                        <button 
                            class=move || if active_filter.get() == "all" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| set_active_filter.set("all".to_string())
                        >
                            "📥 All"
                        </button>
                        <button 
                            class=move || if active_filter.get() == "unread" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| set_active_filter.set("unread".to_string())
                        >
                            "🔴 Unread"
                        </button>
                        <button 
                            class=move || if active_filter.get() == "assigned" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| set_active_filter.set("assigned".to_string())
                        >
                            "👤 Assigned to Me"
                        </button>
                        <button 
                            class=move || if active_filter.get() == "sent" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| set_active_filter.set("sent".to_string())
                        >
                            "📤 Sent"
                        </button>
                    </nav>
                </aside>
                
                
                // Pane 2: Thread List
                <div class="inbox-thread-list">
                    <Suspense fallback=move || view! { <div class="loading">"Loading threads..."</div> }>
                        {move || {
                            threads_resource.get().map(|threads| {
                                let selected = selected_thread_id.get();
                                view! {
                                    <InboxThreadList 
                                        threads=threads
                                        selected_id=selected
                                        on_select=on_select_thread.clone()
                                    />
                                }
                            })
                        }}
                    </Suspense>
                </div>
                
                // Pane 3: Conversation View
                <div class="inbox-conversation">
                    <Show
                        when=move || selected_thread_id.get().is_some()
                        fallback=|| view! {
                            <div class="conversation-empty">
                                <div class="empty-icon">"📬"</div>
                                <p>"Select a conversation to view messages"</p>
                            </div>
                        }
                    >
                        <div class="conversation-container">
                            // Header with back button
                            <header class="conversation-header">
                                <button class="back-btn" on:click=on_back>
                                    "← Back"
                                </button>
                                {move || {
                                    messages_resource.get().flatten().map(|msg_data| {
                                        let entity_type = selected_entity_type.get().unwrap_or_else(|| "contact".to_string());
                                        let entity_id = selected_thread_id.get().unwrap_or_default();
                                        let record_link = format!("/app/{}/entity/{}/{}", 
                                            if entity_type == "contact" { "crm" } else { "crm" },
                                            entity_type,
                                            entity_id
                                        );
                                        view! {
                                            <div class="header-content">
                                                <h2 class="entity-name">{msg_data.entity_name.clone()}</h2>
                                                <a href=record_link class="go-to-record">"Go to Record →"</a>
                                            </div>
                                        }
                                    })
                                }}
                            </header>
                            
                            // Message List
                            <div class="message-list">
                                {move || {
                                    messages_resource.get().flatten().map(|msg_data| {
                                        let messages = msg_data.messages;
                                        view! {
                                            <div class="messages-container">
                                                {messages.into_iter().map(|msg| {
                                                    view! { <MessageBubble message=msg /> }
                                                }).collect_view()}
                                            </div>
                                        }
                                    })
                                }}
                            </div>
                            
                            // Composer
                            <div class="composer-container">
                                <Composer 
                                    content=reply_content
                                    set_content=set_reply_content
                                    message_type=reply_type
                                    set_message_type=set_reply_type
                                    on_send=move || {
                                        let content = reply_content.get();
                                        let msg_type = reply_type.get();
                                        if !content.trim().is_empty() {
                                            if let Some(id) = selected_thread_id.get() {
                                                send_action.dispatch((id, content, msg_type));
                                            }
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
