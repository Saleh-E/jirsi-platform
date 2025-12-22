//! Composer Component - HubSpot-style activity composer
//!
//! Features:
//! - Tabs for Note, Email, Call, Task
//! - Slash command support (/note, /email, /call, /task)
//! - Expandable input area
//! - Saves interactions via API

use leptos::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ComposerTab {
    Note,
    Email,
    Call,
    Task,
}

#[allow(dead_code)]
impl ComposerTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Email => "Email",
            Self::Call => "Call",
            Self::Task => "Task",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Note => "📝",
            Self::Email => "✉️",
            Self::Call => "📞",
            Self::Task => "✓",
        }
    }

    fn interaction_type(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Email => "email",
            Self::Call => "call",
            Self::Task => "task",
        }
    }

    fn placeholder(&self) -> &'static str {
        match self {
            Self::Note => "Write a note... (type /task or /call to switch)",
            Self::Email => "Compose an email...",
            Self::Call => "Log a call summary...",
            Self::Task => "Describe the task...",
        }
    }
}

/// HubSpot-style activity composer with tabs
#[component]
pub fn Composer(
    /// Entity type (contact, company, deal, etc.)
    #[prop(into)] entity_type: String,
    /// Record ID to attach activity to
    #[prop(into)] record_id: String,
    /// Callback when activity is created
    #[prop(optional)] on_activity_added: Option<Callback<()>>,
) -> impl IntoView {
    let _entity_type_stored = store_value(entity_type);
    let _record_id_stored = store_value(record_id);

    // State
    let (active_tab, set_active_tab) = create_signal(ComposerTab::Note);
    let (content, set_content) = create_signal(String::new());
    let (is_expanded, set_is_expanded) = create_signal(false);
    let (is_saving, set_is_saving) = create_signal(false);

    // Handle slash commands in content
    let handle_input = move |ev: web_sys::Event| {
        let value = event_target_value(&ev);

        // Check for slash commands at start of input
        if value.starts_with("/note") {
            set_active_tab.set(ComposerTab::Note);
            set_content.set(value.replace("/note", "").trim().to_string());
        } else if value.starts_with("/email") {
            set_active_tab.set(ComposerTab::Email);
            set_content.set(value.replace("/email", "").trim().to_string());
        } else if value.starts_with("/call") {
            set_active_tab.set(ComposerTab::Call);
            set_content.set(value.replace("/call", "").trim().to_string());
        } else if value.starts_with("/task") {
            set_active_tab.set(ComposerTab::Task);
            set_content.set(value.replace("/task", "").trim().to_string());
        } else {
            set_content.set(value);
        }
    };

    // Submit activity (placeholder - would call API)
    let on_activity_added_stored = store_value(on_activity_added);
    let submit_activity = move |_| {
        let text = content.get();
        if text.trim().is_empty() {
            return;
        }

        set_is_saving.set(true);

        // TODO: Call create_interaction API
        // For now, just simulate success
        set_timeout(
            move || {
                set_content.set(String::new());
                set_is_expanded.set(false);
                set_is_saving.set(false);
                if let Some(cb) = on_activity_added_stored.get_value() {
                    cb.call(());
                }
            },
            std::time::Duration::from_millis(500),
        );
    };

    // Cancel and collapse
    let cancel = move |_| {
        set_content.set(String::new());
        set_is_expanded.set(false);
    };

    view! {
        <div class="composer">
            // Tab row
            <div class="composer-tabs">
                {[ComposerTab::Note, ComposerTab::Email, ComposerTab::Call, ComposerTab::Task]
                    .into_iter()
                    .map(|tab| {
                        let is_active = move || active_tab.get() == tab;
                        view! {
                            <button
                                class=move || format!("composer-tab {}", if is_active() { "active" } else { "" })
                                on:click=move |_| {
                                    set_active_tab.set(tab);
                                    set_is_expanded.set(true);
                                }
                            >
                                <span class="tab-icon">{tab.icon()}</span>
                                <span class="tab-label">{tab.label()}</span>
                            </button>
                        }
                    })
                    .collect_view()
                }
            </div>

            // Input area (expandable)
            <div class=move || format!("composer-body {}", if is_expanded.get() { "expanded" } else { "" })>
                <textarea
                    class="composer-input"
                    placeholder=move || active_tab.get().placeholder()
                    prop:value=move || content.get()
                    on:input=handle_input
                    on:focus=move |_| set_is_expanded.set(true)
                    rows=move || if is_expanded.get() { 4 } else { 1 }
                />

                // Action buttons (only when expanded)
                {move || is_expanded.get().then(|| view! {
                    <div class="composer-actions">
                        <button class="composer-btn cancel" on:click=cancel>
                            "Cancel"
                        </button>
                        <button
                            class="composer-btn submit"
                            on:click=submit_activity
                            disabled=move || content.get().trim().is_empty() || is_saving.get()
                        >
                            {move || if is_saving.get() { "Saving..." } else { "Save" }}
                        </button>
                    </div>
                })}
            </div>
        </div>
    }
}

// Legacy Composer signature for backwards compatibility
#[component]
pub fn LegacyComposer(
    content: ReadSignal<String>,
    set_content: WriteSignal<String>,
    message_type: ReadSignal<String>,
    set_message_type: WriteSignal<String>,
    on_send: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let on_send_clone = on_send.clone();

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && ev.ctrl_key() {
            ev.prevent_default();
            on_send_clone();
        }
    };

    view! {
        <div class="composer legacy">
            <div class="composer-type-selector">
                <button
                    class=move || if message_type.get() == "note" { "type-btn active" } else { "type-btn" }
                    on:click=move |_| set_message_type.set("note".to_string())
                >
                    "📝 Note"
                </button>
                <button
                    class=move || if message_type.get() == "email" { "type-btn active" } else { "type-btn" }
                    on:click=move |_| set_message_type.set("email".to_string())
                >
                    "✉️ Email"
                </button>
            </div>

            <textarea
                class="composer-textarea"
                placeholder="Type your message... (Ctrl+Enter to send)"
                prop:value=move || content.get()
                on:input=move |ev| set_content.set(event_target_value(&ev))
                on:keydown=on_keydown
            />

            <div class="composer-actions">
                <button
                    class="send-btn"
                    on:click=move |_| on_send()
                    disabled=move || content.get().trim().is_empty()
                >
                    "Send"
                </button>
            </div>
        </div>
    }
}
