//! Unified Timeline & Composer Components
//! 
//! The "One Engagement" Engine - Works for ANY entity type.
//! This is the "heart" and "brain" of the record detail page.

use leptos::*;
use crate::api::{fetch_interactions, create_interaction, Interaction};
use crate::utils::format_datetime;

/// Interaction types with their visual properties
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComposerTab {
    Note,
    Email,
    Call,
    Task,
}

impl ComposerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComposerTab::Note => "note",
            ComposerTab::Email => "email",
            ComposerTab::Call => "call",
            ComposerTab::Task => "task",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            ComposerTab::Note => "Note",
            ComposerTab::Email => "Email",
            ComposerTab::Call => "Call",
            ComposerTab::Task => "Task",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            ComposerTab::Note => "📝",
            ComposerTab::Email => "📧",
            ComposerTab::Call => "📞",
            ComposerTab::Task => "✓",
        }
    }
    
    pub fn placeholder(&self) -> &'static str {
        match self {
            ComposerTab::Note => "Write a note about this record...",
            ComposerTab::Email => "Compose your email message...",
            ComposerTab::Call => "Log call notes and outcome...",
            ComposerTab::Task => "Describe the task...",
        }
    }
}

/// The Unified Composer - Tabbed activity input
/// "The Brain" of the record detail page
#[component]
pub fn UnifiedComposer(
    entity_type: String,
    record_id: String,
    #[prop(into)] on_activity_created: Callback<()>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal(ComposerTab::Note);
    let (content, set_content) = create_signal(String::new());
    let (title, set_title) = create_signal(String::new());
    let (duration, set_duration) = create_signal(String::new());
    let (submitting, set_submitting) = create_signal(false);
    let (expanded, set_expanded) = create_signal(false);
    
    // Reset form when tab changes (Optimistic UI - instant switch)
    let handle_tab_change = move |tab: ComposerTab| {
        set_active_tab.set(tab);
        set_content.set(String::new());
        set_title.set(String::new());
        set_duration.set(String::new());
        set_expanded.set(false);
    };
    
    // Submit interaction
    let etype = entity_type.clone();
    let rid = record_id.clone();
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        let tab = active_tab.get();
        let content_val = content.get();
        let title_val = title.get();
        
        if content_val.trim().is_empty() && title_val.trim().is_empty() {
            return;
        }
        
        let etype = etype.clone();
        let rid = rid.clone();
        let on_done = on_activity_created.clone();
        
        spawn_local(async move {
            set_submitting.set(true);
            
            let interaction_type = tab.as_str();
            let final_title = if title_val.is_empty() {
                format!("{} logged", tab.label())
            } else {
                title_val
            };
            
            match create_interaction(
                &etype,
                &rid,
                interaction_type,
                &final_title,
                Some(&content_val),
                "00000000-0000-0000-0000-000000000000", // TODO: Get from auth
            ).await {
                Ok(_) => {
                    set_content.set(String::new());
                    set_title.set(String::new());
                    set_duration.set(String::new());
                    set_expanded.set(false);
                    on_done.call(());
                }
                Err(e) => {
                    logging::log!("Failed to create interaction: {}", e);
                }
            }
            
            set_submitting.set(false);
        });
    };
    
    view! {
        <div class="unified-composer">
            // Tab bar
            <div class="composer-tabs">
                <button
                    class=move || format!("composer-tab {}", if active_tab.get() == ComposerTab::Note { "active" } else { "" })
                    data-type="note"
                    on:click=move |_| handle_tab_change(ComposerTab::Note)
                >
                    <span class="tab-icon">"📝"</span>
                    <span class="tab-label">"Note"</span>
                </button>
                <button
                    class=move || format!("composer-tab {}", if active_tab.get() == ComposerTab::Email { "active" } else { "" })
                    data-type="email"
                    on:click=move |_| handle_tab_change(ComposerTab::Email)
                >
                    <span class="tab-icon">"📧"</span>
                    <span class="tab-label">"Email"</span>
                </button>
                <button
                    class=move || format!("composer-tab {}", if active_tab.get() == ComposerTab::Call { "active" } else { "" })
                    data-type="call"
                    on:click=move |_| handle_tab_change(ComposerTab::Call)
                >
                    <span class="tab-icon">"📞"</span>
                    <span class="tab-label">"Call"</span>
                </button>
                <button
                    class=move || format!("composer-tab {}", if active_tab.get() == ComposerTab::Task { "active" } else { "" })
                    data-type="task"
                    on:click=move |_| handle_tab_change(ComposerTab::Task)
                >
                    <span class="tab-icon">"✓"</span>
                    <span class="tab-label">"Task"</span>
                </button>
            </div>
            
            // Form body (changes based on tab)
            <form class="composer-body" on:submit=handle_submit>
                // Title field for Email/Call/Task
                {move || (active_tab.get() != ComposerTab::Note).then(|| {
                    let tab = active_tab.get();
                    view! {
                        <div class="composer-field-row">
                            <div class="composer-field">
                                <label class="composer-field-label">
                                    {if tab == ComposerTab::Email { "Subject" } else { "Title" }}
                                </label>
                                <input
                                    type="text"
                                    class="composer-field-input"
                                    placeholder=move || format!("{} {}", tab.label(), if tab == ComposerTab::Email { "subject" } else { "title" })
                                    prop:value=title
                                    on:input=move |ev| set_title.set(event_target_value(&ev))
                                />
                            </div>
                            // Duration for calls
                            {(tab == ComposerTab::Call).then(|| view! {
                                <div class="composer-field" style="max-width: 120px;">
                                    <label class="composer-field-label">"Duration"</label>
                                    <input
                                        type="text"
                                        class="composer-field-input"
                                        placeholder="e.g. 15 min"
                                        prop:value=duration
                                        on:input=move |ev| set_duration.set(event_target_value(&ev))
                                    />
                                </div>
                            })}
                        </div>
                    }
                })}
                
                // Main content input
                <textarea
                    class="composer-input"
                    placeholder=move || active_tab.get().placeholder()
                    prop:value=content
                    on:input=move |ev| set_content.set(event_target_value(&ev))
                    on:focus=move |_| set_expanded.set(true)
                    rows=move || if expanded.get() { 4 } else { 2 }
                ></textarea>
                
                // Actions
                {move || expanded.get().then(|| view! {
                    <div class="composer-actions">
                        <div class="composer-actions-left">
                            // Future: attachments, @mentions
                        </div>
                        <div class="composer-actions-right">
                            <button
                                type="button"
                                class="composer-btn cancel"
                                on:click=move |_| {
                                    set_expanded.set(false);
                                    set_content.set(String::new());
                                }
                            >
                                "Cancel"
                            </button>
                            <button
                                type="submit"
                                class="composer-btn submit"
                                disabled=move || submitting.get() || content.get().trim().is_empty()
                            >
                                {move || if submitting.get() { "Saving..." } else { 
                                    match active_tab.get() {
                                        ComposerTab::Note => "Save Note",
                                        ComposerTab::Email => "Log Email",
                                        ComposerTab::Call => "Log Call",
                                        ComposerTab::Task => "Create Task",
                                    }
                                }}
                            </button>
                        </div>
                    </div>
                })}
            </form>
        </div>
    }
}

/// The Unified Timeline - Color-coded activity stream
/// "The Heart" of the record detail page
/// 
/// This component works for ANY entity type (Contact, Property, Deal, etc.)
#[component]
pub fn UnifiedTimeline(
    entity_type: String,
    record_id: String,
    #[prop(default = create_rw_signal(0))]
    reload_trigger: RwSignal<u32>,
) -> impl IntoView {
    let (interactions, set_interactions) = create_signal(Vec::<Interaction>::new());
    let (loading, set_loading) = create_signal(true);
    
    // Fetch interactions on mount and when reload_trigger changes
    let etype = entity_type.clone();
    let rid = record_id.clone();
    create_effect(move |_| {
        let etype = etype.clone();
        let rid = rid.clone();
        let _ = reload_trigger.get(); // Subscribe to reload
        
        spawn_local(async move {
            set_loading.set(true);
            match fetch_interactions(&etype, &rid).await {
                Ok(response) => set_interactions.set(response.data),
                Err(e) => logging::log!("Failed to fetch interactions: {}", e),
            }
            set_loading.set(false);
        });
    });
    
    view! {
        <div class="timeline-container">
            // Loading state
            {move || loading.get().then(|| view! {
                <div class="timeline-list">
                    // Skeleton items
                    {(0..3).map(|_| view! {
                        <div class="timeline-item">
                            <div class="timeline-icon skeleton" style="width: 24px; height: 24px;"></div>
                            <div class="timeline-content skeleton" style="height: 80px;"></div>
                        </div>
                    }).collect_view()}
                </div>
            })}
            
            // Empty state
            {move || (!loading.get() && interactions.get().is_empty()).then(|| view! {
                <div class="timeline-empty">
                    <div class="timeline-empty-icon">"💬"</div>
                    <h4 class="timeline-empty-title">"No activities yet"</h4>
                    <p class="timeline-empty-desc">
                        "Log a note, call, or email to start tracking engagement."
                    </p>
                </div>
            })}
            
            // Timeline list
            {move || (!loading.get() && !interactions.get().is_empty()).then(|| {
                let items = interactions.get();
                view! {
                    <div class="timeline-list">
                        {items.into_iter().map(|interaction| {
                            let itype = interaction.interaction_type.to_lowercase();
                            let item_class = format!("timeline-item {}", itype);
                            let icon = get_interaction_icon(&itype);
                            let formatted_date = format_datetime(&interaction.occurred_at);
                            
                            view! {
                                <div class=item_class>
                                    <div class="timeline-icon">
                                        {icon}
                                    </div>
                                    <div class="timeline-content">
                                        <div class="timeline-content-header">
                                            <h5 class="timeline-title">{interaction.title}</h5>
                                            <span class="timeline-meta">{formatted_date}</span>
                                        </div>
                                        {interaction.content.map(|c| view! {
                                            <p class="timeline-body">{c}</p>
                                        })}
                                        {interaction.duration_minutes.map(|mins| view! {
                                            <div class="timeline-footer">
                                                <span class="timeline-duration">
                                                    "⏱️ " {mins} " min"
                                                </span>
                                            </div>
                                        })}
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }
            })}
        </div>
    }
}

/// Get emoji icon for interaction type
fn get_interaction_icon(itype: &str) -> &'static str {
    match itype {
        "call" => "📞",
        "email" => "📧",
        "task" => "✓",
        "meeting" => "📅",
        "message" => "💬",
        _ => "📝", // note default
    }
}
