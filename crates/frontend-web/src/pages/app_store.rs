//! App Store Marketplace Page
//!
//! Provides:
//! - App discovery with categories
//! - Search and filtering
//! - App detail view
//! - Installation flow

use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// App listing from marketplace
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppListing {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub publisher_name: String,
    pub version: String,
    pub rating: Option<f32>,
    pub downloads: i32,
    pub category: String,
    pub is_installed: bool,
}

/// App category
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppCategory {
    pub code: String,
    pub name: String,
    pub icon: String,
    pub description: String,
}

/// App Store Page Component
#[component]
pub fn AppStorePage() -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (selected_category, set_selected_category) = create_signal(Option::<String>::None);
    let (selected_app, set_selected_app) = create_signal(Option::<AppListing>::None);
    let (installing, set_installing) = create_signal(false);
    
    // Categories
    let categories = vec![
        AppCategory { code: "all".to_string(), name: "All Apps".to_string(), icon: "📱".to_string(), description: "Browse all apps".to_string() },
        AppCategory { code: "crm".to_string(), name: "CRM & Sales".to_string(), icon: "💼".to_string(), description: "Sales tools".to_string() },
        AppCategory { code: "real_estate".to_string(), name: "Real Estate".to_string(), icon: "🏠".to_string(), description: "Property management".to_string() },
        AppCategory { code: "marketing".to_string(), name: "Marketing".to_string(), icon: "📣".to_string(), description: "Marketing automation".to_string() },
        AppCategory { code: "finance".to_string(), name: "Finance".to_string(), icon: "💰".to_string(), description: "Payments & invoicing".to_string() },
        AppCategory { code: "analytics".to_string(), name: "Analytics".to_string(), icon: "📊".to_string(), description: "Business intelligence".to_string() },
    ];
    
    // Sample apps
    let apps = create_resource(
        move || (search_query.get(), selected_category.get()),
        |_| async move {
            vec![
                AppListing {
                    id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
                    code: "crm".to_string(),
                    name: "Jirsi CRM".to_string(),
                    icon: "💼".to_string(),
                    description: "Complete customer relationship management with contacts, deals, and pipeline management.".to_string(),
                    publisher_name: "Jirsi".to_string(),
                    version: "2.0.0".to_string(),
                    rating: Some(5.0),
                    downloads: 1250,
                    category: "crm".to_string(),
                    is_installed: true,
                },
                AppListing {
                    id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
                    code: "real_estate".to_string(),
                    name: "Jirsi Real Estate".to_string(),
                    icon: "🏠".to_string(),
                    description: "Property management, listings, viewings, and contract lifecycle management.".to_string(),
                    publisher_name: "Jirsi".to_string(),
                    version: "2.0.0".to_string(),
                    rating: Some(4.8),
                    downloads: 890,
                    category: "real_estate".to_string(),
                    is_installed: true,
                },
                AppListing {
                    id: Uuid::new_v4(),
                    code: "email_campaigns".to_string(),
                    name: "Email Campaigns Pro".to_string(),
                    icon: "📧".to_string(),
                    description: "Create and send beautiful email campaigns with analytics and A/B testing.".to_string(),
                    publisher_name: "MailPro".to_string(),
                    version: "1.5.0".to_string(),
                    rating: Some(4.6),
                    downloads: 520,
                    category: "marketing".to_string(),
                    is_installed: false,
                },
                AppListing {
                    id: Uuid::new_v4(),
                    code: "invoice_wizard".to_string(),
                    name: "Invoice Wizard".to_string(),
                    icon: "📄".to_string(),
                    description: "Professional invoicing with automatic reminders and payment tracking.".to_string(),
                    publisher_name: "FinTools".to_string(),
                    version: "3.2.1".to_string(),
                    rating: Some(4.9),
                    downloads: 3200,
                    category: "finance".to_string(),
                    is_installed: false,
                },
                AppListing {
                    id: Uuid::new_v4(),
                    code: "lead_scoring".to_string(),
                    name: "AI Lead Scoring".to_string(),
                    icon: "🎯".to_string(),
                    description: "Automatically score and prioritize leads using machine learning.".to_string(),
                    publisher_name: "Jirsi".to_string(),
                    version: "1.0.0".to_string(),
                    rating: Some(4.7),
                    downloads: 180,
                    category: "crm".to_string(),
                    is_installed: false,
                },
            ]
        }
    );
    
    let install_app = move |app: AppListing| {
        set_installing.set(true);
        spawn_local(async move {
            // Simulate installation
            gloo_timers::future::TimeoutFuture::new(2000).await;
            set_installing.set(false);
            set_selected_app.set(None);
        });
    };
    
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-6">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-white mb-2">"App Marketplace"</h1>
                <p class="text-slate-400">"Discover apps and integrations to enhance your workflow"</p>
            </div>
            
            // Search bar
            <div class="mb-8">
                <div class="relative max-w-xl">
                    <span class="absolute left-4 top-1/2 -translate-y-1/2 text-slate-500">"🔍"</span>
                    <input
                        type="text"
                        class="w-full bg-white/5 border border-white/10 rounded-xl pl-12 pr-4 py-3 text-white placeholder-slate-500 focus:border-indigo-500 focus:outline-none"
                        placeholder="Search apps..."
                        prop:value=search_query
                        on:input=move |e| set_search_query.set(event_target_value(&e))
                    />
                </div>
            </div>
            
            // Categories
            <div class="flex gap-3 mb-8 overflow-x-auto pb-2">
                {categories.into_iter().map(|cat| {
                    let code = cat.code.clone();
                    let is_selected = move || {
                        selected_category.get().as_deref() == Some(&code) || 
                        (code == "all" && selected_category.get().is_none())
                    };
                    view! {
                        <button
                            class="flex items-center gap-2 px-4 py-2 rounded-full whitespace-nowrap transition-all"
                            class:bg-indigo-500=is_selected
                            class:text-white=is_selected
                            class:bg-white/5=move || !is_selected()
                            class:text-slate-400=move || !is_selected()
                            class:hover:bg-white/10=move || !is_selected()
                            on:click=move |_| {
                                if cat.code == "all" {
                                    set_selected_category.set(None);
                                } else {
                                    set_selected_category.set(Some(cat.code.clone()));
                                }
                            }
                        >
                            <span>{cat.icon}</span>
                            <span>{cat.name}</span>
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            // App grid
            <Suspense fallback=move || view! { <div class="text-slate-400">"Loading apps..."</div> }>
                {move || apps.get().map(|apps_list| {
                    let filtered: Vec<_> = apps_list.into_iter()
                        .filter(|app| {
                            let matches_category = selected_category.get()
                                .map(|c| app.category == c)
                                .unwrap_or(true);
                            let matches_search = search_query.get().is_empty() ||
                                app.name.to_lowercase().contains(&search_query.get().to_lowercase());
                            matches_category && matches_search
                        })
                        .collect();
                    
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                            {filtered.into_iter().map(|app| {
                                let app_clone = app.clone();
                                view! {
                                    <AppCard 
                                        app=app 
                                        on_click=move |a| set_selected_app.set(Some(a))
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                })}
            </Suspense>
            
            // App detail modal
            {move || selected_app.get().map(|app| {
                view! {
                    <AppDetailModal 
                        app=app
                        installing=installing
                        on_install=install_app
                        on_close=move |_| set_selected_app.set(None)
                    />
                }
            })}
        </div>
    }
}

/// App Card Component
#[component]
fn AppCard(
    app: AppListing,
    on_click: impl Fn(AppListing) + 'static,
) -> impl IntoView {
    let app_clone = app.clone();
    
    view! {
        <button
            class="bg-white/5 border border-white/10 rounded-2xl p-6 text-left hover:bg-white/10 hover:border-indigo-500/50 transition-all group"
            on:click=move |_| on_click(app_clone.clone())
        >
            <div class="flex items-start gap-4 mb-4">
                <div class="text-4xl">{&app.icon}</div>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-white truncate">{&app.name}</h3>
                    <p class="text-sm text-slate-500">{&app.publisher_name}</p>
                </div>
                {app.is_installed.then(|| view! {
                    <span class="px-2 py-1 bg-green-500/20 text-green-400 text-xs rounded-full">"Installed"</span>
                })}
            </div>
            
            <p class="text-sm text-slate-400 mb-4 line-clamp-2">{&app.description}</p>
            
            <div class="flex items-center justify-between text-sm">
                <div class="flex items-center gap-1 text-yellow-400">
                    "⭐"
                    <span class="text-white">{app.rating.map(|r| format!("{:.1}", r)).unwrap_or("-".to_string())}</span>
                </div>
                <span class="text-slate-500">{format!("{} installs", app.downloads)}</span>
            </div>
        </button>
    }
}

/// App Detail Modal Component
#[component]
fn AppDetailModal(
    app: AppListing,
    installing: ReadSignal<bool>,
    on_install: impl Fn(AppListing) + 'static + Clone,
    on_close: impl Fn(()) + 'static,
) -> impl IntoView {
    let app_clone = app.clone();
    let on_install_clone = on_install.clone();
    
    view! {
        <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
            <div class="bg-slate-900 border border-white/10 rounded-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
                // Header
                <div class="sticky top-0 bg-slate-900 border-b border-white/10 p-6 flex items-start gap-4">
                    <div class="text-5xl">{&app.icon}</div>
                    <div class="flex-1">
                        <h2 class="text-2xl font-bold text-white">{&app.name}</h2>
                        <p class="text-slate-400">{format!("by {} • v{}", app.publisher_name, app.version)}</p>
                        <div class="flex items-center gap-4 mt-2 text-sm">
                            <span class="flex items-center gap-1 text-yellow-400">
                                "⭐ "
                                <span class="text-white">{app.rating.map(|r| format!("{:.1}", r)).unwrap_or("-".to_string())}</span>
                            </span>
                            <span class="text-slate-500">{format!("{} installs", app.downloads)}</span>
                        </div>
                    </div>
                    <button
                        class="text-2xl text-slate-500 hover:text-white"
                        on:click=move |_| on_close(())
                    >
                        "✕"
                    </button>
                </div>
                
                // Body
                <div class="p-6">
                    <h3 class="text-lg font-semibold text-white mb-3">"About"</h3>
                    <p class="text-slate-400 mb-6">{&app.description}</p>
                    
                    // Permissions
                    <h3 class="text-lg font-semibold text-white mb-3">"Permissions Required"</h3>
                    <ul class="space-y-2 mb-6">
                        <li class="flex items-center gap-2 text-slate-400">
                            <span class="text-green-400">"✓"</span>
                            "Read and write contacts"
                        </li>
                        <li class="flex items-center gap-2 text-slate-400">
                            <span class="text-green-400">"✓"</span>
                            "Access workflow automation"
                        </li>
                        <li class="flex items-center gap-2 text-slate-400">
                            <span class="text-green-400">"✓"</span>
                            "Send notifications"
                        </li>
                    </ul>
                    
                    // Install button
                    {if app.is_installed {
                        view! {
                            <div class="flex gap-3">
                                <button class="flex-1 py-3 bg-slate-700 text-white rounded-lg font-medium">
                                    "Open App"
                                </button>
                                <button class="px-6 py-3 text-red-400 hover:text-red-300 rounded-lg font-medium">
                                    "Uninstall"
                                </button>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <button
                                class="w-full py-3 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-lg font-medium hover:shadow-lg hover:shadow-indigo-500/25 transition-all disabled:opacity-50"
                                disabled=installing
                                on:click=move |_| on_install_clone(app_clone.clone())
                            >
                                {move || if installing.get() { "Installing..." } else { "Install App" }}
                            </button>
                        }.into_view()
                    }}
                </div>
            </div>
        </div>
    }
}

/// App Settings Panel Component
#[component]
pub fn AppSettingsPanel(app_id: Uuid) -> impl IntoView {
    let (settings, set_settings) = create_signal(serde_json::json!({
        "auto_sync": true,
        "notification_level": "all",
        "custom_field": ""
    }));
    let (saving, set_saving) = create_signal(false);
    
    let save_settings = move |_| {
        set_saving.set(true);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1000).await;
            set_saving.set(false);
        });
    };
    
    view! {
        <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
            <h3 class="text-xl font-bold text-white mb-6">"App Settings"</h3>
            
            <div class="space-y-6">
                // Auto sync toggle
                <div class="flex items-center justify-between">
                    <div>
                        <p class="font-medium text-white">"Auto Sync"</p>
                        <p class="text-sm text-slate-400">"Automatically sync data in the background"</p>
                    </div>
                    <button
                        class="relative w-12 h-6 rounded-full bg-indigo-500 transition-colors"
                    >
                        <div class="absolute top-1 left-7 w-4 h-4 bg-white rounded-full transition-all" />
                    </button>
                </div>
                
                // Notification level
                <div>
                    <label class="block font-medium text-white mb-2">"Notification Level"</label>
                    <select class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white">
                        <option value="all">"All notifications"</option>
                        <option value="important">"Important only"</option>
                        <option value="none">"None"</option>
                    </select>
                </div>
                
                // Save button
                <button
                    class="w-full py-3 bg-indigo-500 text-white rounded-lg font-medium hover:bg-indigo-600 transition-colors disabled:opacity-50"
                    disabled=saving
                    on:click=save_settings
                >
                    {move || if saving.get() { "Saving..." } else { "Save Settings" }}
                </button>
            </div>
        </div>
    }
}
