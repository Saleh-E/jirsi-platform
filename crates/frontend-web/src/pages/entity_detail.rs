//! Cinematic Entity Detail - Full Feature Edition
//! Metadata driven + Glassmorphism + Activity (Composer + Timeline) + Audit Timeline + Tabs

use leptos::*;
use leptos_router::*;
use uuid::Uuid;
use crate::api::{
    fetch_field_defs, fetch_entity, update_entity, FieldDef
};
use crate::design_system::inputs::smart_field::SmartField;
use crate::components::audit_timeline::AuditTimeline;
use crate::components::timeline::{UnifiedComposer, UnifiedTimeline};

#[component]
pub fn EntityDetailPage() -> impl IntoView {
    let params = use_params_map();
    let entity_type = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    let record_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());
    
    // Signals
    let (fields, set_fields) = create_signal(Vec::<FieldDef>::new());
    let (record, set_record) = create_signal(serde_json::Value::Null);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);
    
    // Tab State - Now with 4 tabs: Details, Activity, Related, Audit
    let (active_tab, set_active_tab) = create_signal("details".to_string());
    
    // Activity reload trigger
    let reload_activity = create_rw_signal(0u32);
    
    // Load Data
    let entity_for_load = entity_type.clone();
    let id_for_load = record_id.clone();
    
    create_effect(move |_| {
        let etype = entity_for_load();
        let id = id_for_load();
        if etype.is_empty() || id.is_empty() { return; }
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            // Fetch Defs
            if let Ok(f) = fetch_field_defs(&etype).await {
                set_fields.set(f);
            }
            
            // Fetch Record
            match fetch_entity(&etype, &id).await {
                Ok(r) => set_record.set(r),
                Err(e) => set_error.set(Some(e)),
            }
            
            set_loading.set(false);
        });
    });

    // Handler for activity creation
    let on_activity_created = Callback::new(move |_| {
        reload_activity.update(|n| *n += 1);
    });

    view! {
        <div class="h-full flex flex-col p-8 overflow-y-auto custom-scrollbar animate-fade-in">
            // Header with Entity Title
            <header class="mb-6">
                <div class="flex items-center justify-between">
                    <div>
                        <a href=move || format!("/app/crm/entity/{}", entity_type()) 
                           class="text-sm text-zinc-400 hover:text-violet-400 transition-colors mb-2 inline-flex items-center gap-2">
                            <i class="fa-solid fa-arrow-left text-xs"></i>
                            "Back to " {move || entity_type().to_uppercase()}
                        </a>
                        <h1 class="text-3xl font-bold text-white tracking-tight mt-1">
                            {move || {
                                let r = record.get();
                                let first = r.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
                                let last = r.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
                                let name = r.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                
                                if !first.is_empty() { format!("{} {}", first, last) }
                                else if !name.is_empty() { name.to_string() }
                                else if !title.is_empty() { title.to_string() }
                                else { "Unnamed Record".to_string() }
                            }}
                        </h1>
                        // Show entity type as badge
                        <span class="inline-flex items-center mt-2 px-3 py-1 rounded-full text-xs font-medium bg-violet-500/10 text-violet-400 border border-violet-500/20">
                            {move || entity_type().to_uppercase()}
                        </span>
                    </div>
                    
                    // Action buttons
                    <div class="flex gap-2">
                        <button class="ui-btn ui-btn-secondary">
                            <i class="fa-solid fa-share-nodes mr-2"></i>
                            "Share"
                        </button>
                        <button class="ui-btn ui-btn-primary">
                            <i class="fa-solid fa-edit mr-2"></i>
                            "Edit"
                        </button>
                    </div>
                </div>
            </header>
            
            // Enhanced Tabs with icons
            <div class="ui-tabs">
                <button 
                    class=move || format!("ui-tab {}", 
                        if active_tab.get() == "details" { "active" } else { "" }
                    )
                    on:click=move |_| set_active_tab.set("details".to_string())
                >
                    <i class="fa-solid fa-list text-xs"></i>
                    "Details"
                </button>
                <button 
                    class=move || format!("ui-tab {}", 
                        if active_tab.get() == "activity" { "active" } else { "" }
                    )
                    on:click=move |_| set_active_tab.set("activity".to_string())
                >
                    <i class="fa-solid fa-comments text-xs"></i>
                    "Activity"
                </button>
                <button 
                    class=move || format!("ui-tab {}", 
                        if active_tab.get() == "related" { "active" } else { "" }
                    )
                    on:click=move |_| set_active_tab.set("related".to_string())
                >
                    <i class="fa-solid fa-link text-xs"></i>
                    "Related"
                </button>
                <button 
                    class=move || format!("ui-tab {}", 
                        if active_tab.get() == "audit" { "active" } else { "" }
                    )
                    on:click=move |_| set_active_tab.set("audit".to_string())
                >
                    <i class="fa-solid fa-history text-xs"></i>
                    "Audit Trail"
                </button>
            </div>
            
            // Tab Content
            <div class="flex-1">
                {move || match active_tab.get().as_str() {
                    // ===== DETAILS TAB =====
                    "details" => view! {
                        <div class="glass-morphism rounded-2xl p-8 light-edge">
                            {move || if loading.get() {
                                view! { <div class="text-zinc-500 animate-pulse">"Loading fields..."</div> }.into_view()
                            } else if let Some(err) = error.get() {
                                view! { <div class="text-red-400">{err}</div> }.into_view()
                            } else {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                        {move || fields.get().into_iter().map(|field| {
                                            let r = record.get();
                                            let val = r.get(&field.name).cloned().unwrap_or(serde_json::Value::Null);
                                            let field_def = field.clone();
                                            
                                            let etype = entity_type();
                                            let id = record_id();
                                            let fname = field.name.clone();
                                            
                                            let on_change = Callback::new(move |new_val: serde_json::Value| {
                                                let etype = etype.clone();
                                                let id = id.clone();
                                                let fname = fname.clone();
                                                spawn_local(async move {
                                                    let body = serde_json::json!({ fname: new_val });
                                                    let _ = update_entity(&etype, &id, body).await;
                                                });
                                            });

                                            view! {
                                                <div class="glass-field-wrapper p-4 rounded-xl border border-white/5 hover:border-violet-500/30 transition-all group">
                                                    <label class="text-[10px] uppercase tracking-widest text-zinc-500 font-bold mb-2 block group-hover:text-violet-400 transition-colors">
                                                        {field.label.clone()}
                                                    </label>
                                                    <div class="text-sm font-medium text-zinc-100">
                                                        <crate::components::field_renderer::EditableFieldValue
                                                            field={field_def}
                                                            value={val}
                                                            on_change={on_change}
                                                        />
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }}
                        </div>
                    }.into_view(),
                    
                    // ===== ACTIVITY TAB (Composer + Timeline) =====
                    "activity" => {
                         let etype = entity_type();
                         let rid = record_id();
                         let etype2 = etype.clone();
                         let rid2 = rid.clone();
                         
                         view! {
                            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                                // Left Column: Composer
                                <div class="lg:col-span-1">
                                    <div class="glass-morphism rounded-2xl p-6 light-edge sticky top-0">
                                        <h3 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                                            <i class="fa-solid fa-plus-circle text-violet-400"></i>
                                            "Log Activity"
                                        </h3>
                                        <UnifiedComposer 
                                            entity_type=etype.clone()
                                            record_id=rid.clone()
                                            on_activity_created=on_activity_created
                                        />
                                    </div>
                                </div>
                                
                                // Right Column: Timeline
                                <div class="lg:col-span-2">
                                    <div class="glass-morphism rounded-2xl p-6 light-edge">
                                        <h3 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                                            <i class="fa-solid fa-stream text-violet-400"></i>
                                            "Activity History"
                                        </h3>
                                        <UnifiedTimeline 
                                            entity_type=etype2
                                            record_id=rid2
                                            reload_trigger=reload_activity
                                        />
                                    </div>
                                </div>
                            </div>
                         }.into_view()
                    },
                    
                    // ===== RELATED TAB =====
                    "related" => {
                         let etype = entity_type();
                         view! {
                            <div class="glass-morphism rounded-2xl p-8 light-edge">
                                <div class="flex items-center justify-between mb-6">
                                    <h3 class="text-lg font-semibold text-white flex items-center gap-2">
                                        <i class="fa-solid fa-link text-violet-400"></i>
                                        "Related Records"
                                    </h3>
                                    <button class="px-4 py-2 rounded-xl bg-violet-600/10 border border-violet-500/20 text-violet-400 hover:bg-violet-600/20 transition-all text-sm">
                                        <i class="fa-solid fa-plus mr-2"></i>
                                        "Link Record"
                                    </button>
                                </div>
                                
                                // Related entity sections based on entity type
                                <div class="space-y-6">
                                    {move || {
                                        let et = etype.clone();
                                        match et.as_str() {
                                            "contact" => view! {
                                                <div class="space-y-4">
                                                    <RelatedSection title="Properties" icon="fa-building" entity_type="property" />
                                                    <RelatedSection title="Deals" icon="fa-handshake" entity_type="deal" />
                                                    <RelatedSection title="Tasks" icon="fa-check-circle" entity_type="task" />
                                                </div>
                                            }.into_view(),
                                            "property" => view! {
                                                <div class="space-y-4">
                                                    <RelatedSection title="Contacts (Buyers)" icon="fa-users" entity_type="contact" />
                                                    <RelatedSection title="Deals" icon="fa-handshake" entity_type="deal" />
                                                    <RelatedSection title="Tasks" icon="fa-check-circle" entity_type="task" />
                                                </div>
                                            }.into_view(),
                                            "deal" => view! {
                                                <div class="space-y-4">
                                                    <RelatedSection title="Contacts" icon="fa-users" entity_type="contact" />
                                                    <RelatedSection title="Properties" icon="fa-building" entity_type="property" />
                                                    <RelatedSection title="Tasks" icon="fa-check-circle" entity_type="task" />
                                                </div>
                                            }.into_view(),
                                            _ => view! {
                                                <div class="text-zinc-500 text-center py-8">
                                                    "No related entities configured for this type."
                                                </div>
                                            }.into_view()
                                        }
                                    }}
                                </div>
                            </div>
                         }.into_view()
                    },
                    
                    // ===== AUDIT TRAIL TAB =====
                    "audit" => {
                         let id_str = record_id();
                         let etype = entity_type();
                         let uuid = Uuid::parse_str(&id_str).unwrap_or_default();
                         
                         view! {
                            <div class="glass-morphism rounded-2xl p-8 light-edge">
                                <AuditTimeline entity_id=uuid entity_type=etype />
                            </div>
                         }.into_view()
                    },
                    
                    _ => view!{}.into_view()
                }}
            </div>
        </div>
    }
}

/// Related Section Component - Shows related entities in a card
#[component]
fn RelatedSection(
    title: &'static str,
    icon: &'static str,
    entity_type: &'static str,
) -> impl IntoView {
    view! {
        <div class="p-4 rounded-xl bg-white/5 border border-white/5 hover:border-violet-500/20 transition-all">
            <div class="flex items-center justify-between mb-3">
                <h4 class="text-sm font-semibold text-zinc-300 flex items-center gap-2">
                    <i class=format!("fa-solid {} text-zinc-500", icon)></i>
                    {title}
                </h4>
                <a href=format!("/app/crm/entity/{}", entity_type) class="text-xs text-violet-400 hover:text-violet-300">
                    "View All →"
                </a>
            </div>
            <div class="text-zinc-500 text-sm py-4 text-center border border-dashed border-white/10 rounded-lg">
                <i class="fa-solid fa-plus-circle text-lg mb-2 block opacity-50"></i>
                "No related " {title.to_lowercase()} " yet"
            </div>
        </div>
    }
}

#[component]
fn FieldWrapper(
    label: String,
    initial_value: String,
    entity_type: String,
    record_id: String,
    field_name: String, 
) -> impl IntoView {
    let value_signal = create_rw_signal(initial_value);
    
    // Save handler
    let on_save = Callback::new(move |new_val: String| {
        let etype = entity_type.clone();
        let id = record_id.clone();
        let fname = field_name.clone();
        
        spawn_local(async move {
            let body = serde_json::json!({ fname: new_val });
            let _ = update_entity(&etype, &id, body).await;
        });
    });

    view! {
        <SmartField
            value=value_signal
            label={label}
            on_save=on_save
        />
    }
}
