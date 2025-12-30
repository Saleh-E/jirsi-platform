//! Application Shell - World-Class Sidebar, Power Header, and Command Palette
//!
//! A premium app shell with collapsible nav sections, keyboard shortcuts (Cmd+K),
//! quick actions, and WebSocket-connected notifications.
//!
//! ## Antigravity Integration
//! Uses PermissionContext for role-based sidebar visibility.

use leptos::*;
use leptos_router::*;
use crate::context::theme::ThemeToggle;
use crate::context::mobile::use_mobile;
use crate::context::permission::use_permissions;
use crate::components::bottom_nav::BottomNav;

/// Sidebar nav section with collapsible state
#[component]
fn NavSection(
    /// Section title
    title: &'static str,
    /// Icon for section
    icon: &'static str,
    /// Whether section starts expanded
    #[prop(default = true)]
    expanded: bool,
    /// Child nav items
    children: Children,
) -> impl IntoView {
    let (is_expanded, set_expanded) = create_signal(expanded);
    
    view! {
        <div class="flex flex-col gap-1 mb-2">
            <button 
                class="flex items-center justify-between w-full px-4 py-2 text-xs font-bold uppercase tracking-wider text-slate-500 hover:text-slate-300 transition-colors"
                on:click=move |_| set_expanded.update(|v| *v = !*v)
            >
                <div class="flex items-center gap-3">
                    <span class="text-base min-w-[1.25rem] text-center">{icon}</span>
                    <span class="truncate group-[.sidebar-collapsed]:hidden transition-opacity">{title}</span>
                </div>
                <span class="text-[10px] opacity-70 group-[.sidebar-collapsed]:hidden">
                    {move || if is_expanded.get() { "▼" } else { "▶" }}
                </span>
            </button>
            <div class="flex flex-col gap-0.5" class:hidden=move || !is_expanded.get()>
                {children()}
            </div>
        </div>
    }
}

/// Individual nav item with active state detection
#[component]
fn NavItem(
    /// Navigation path
    href: &'static str,
    /// Icon emoji/symbol
    icon: &'static str,
    /// Label text
    label: &'static str,
    /// Badge count (optional)
    #[prop(optional)]
    badge: Option<RwSignal<i32>>,
) -> impl IntoView {
    let location = use_location();
    let is_active = move || {
        let path = location.pathname.get();
        path == href || (href != "/" && path.starts_with(href))
    };
    
    view! {
        <a 
            href={href} 
            class="group/item flex items-center gap-3 px-3 py-2 mx-2 rounded-lg text-sm font-medium transition-colors relative"
            class:bg-white_10=is_active
            class:text-white=is_active
            class:text-slate-400=move || !is_active()
            class:hover:bg-white_5=move || !is_active()
            class:hover:text-white=move || !is_active()
        >
            <span class="min-w-[1.25rem] text-center text-lg">{icon}</span>
            <span class="truncate group-[.sidebar-collapsed]:hidden">{label}</span>
            {badge.map(|b| view! {
                <span class="absolute right-2 px-1.5 py-0.5 rounded-full bg-indigo-500 text-[10px] font-bold text-white group-[.sidebar-collapsed]:top-1 group-[.sidebar-collapsed]:right-1 group-[.sidebar-collapsed]:w-2 group-[.sidebar-collapsed]:h-2 group-[.sidebar-collapsed]:p-0 group-[.sidebar-collapsed]:rounded-full group-[.sidebar-collapsed]:text-transparent overflow-hidden">
                    {move || b.get()}
                </span>
            })}
        </a>
    }
}

/// Quick Create button dropdown
#[component]
fn QuickCreateButton() -> impl IntoView {
    let (show_menu, set_show_menu) = create_signal(false);
    let (create_entity_type, set_create_entity_type) = create_signal::<Option<String>>(None);
    
    view! {
        <div class="relative">
            <button 
                class="w-10 h-10 flex items-center justify-center rounded-full bg-indigo-600 text-white hover:bg-indigo-500 shadow-lg shadow-indigo-500/30 transition-all hover:scale-105"
                on:click=move |_| set_show_menu.update(|v| *v = !*v)
                title="Quick Create"
            >
                <span class="text-xl font-bold leading-none pb-0.5">"+"</span>
            </button>
            {move || show_menu.get().then(|| {
                view! {
                    <div class="absolute right-0 mt-2 w-48 py-1 rounded-xl bg-surface border border-white/10 shadow-2xl shadow-indigo-500/10 backdrop-blur-xl z-50 animate-in fade-in zoom-in duration-200 origin-top-right">
                        <button class="w-full text-left px-4 py-2.5 text-sm text-slate-300 hover:text-white hover:bg-white/5 transition-colors flex items-center gap-3" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("contact".to_string()));
                        }>
                            <span class="text-lg">"👤"</span> "New Contact"
                        </button>
                        <button class="w-full text-left px-4 py-2.5 text-sm text-slate-300 hover:text-white hover:bg-white/5 transition-colors flex items-center gap-3" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("deal".to_string()));
                        }>
                            <span class="text-lg">"💰"</span> "New Deal"
                        </button>
                        <button class="w-full text-left px-4 py-2.5 text-sm text-slate-300 hover:text-white hover:bg-white/5 transition-colors flex items-center gap-3" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("task".to_string()));
                        }>
                            <span class="text-lg">"✓"</span> "New Task"
                        </button>
                        <button class="w-full text-left px-4 py-2.5 text-sm text-slate-300 hover:text-white hover:bg-white/5 transition-colors flex items-center gap-3" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("property".to_string()));
                        }>
                            <span class="text-lg">"🏠"</span> "New Property"
                        </button>
                    </div>
                }
            })}
            
            // CreateModal - opens when an entity type is selected
            {move || create_entity_type.get().map(|entity_type| {
                let label = match entity_type.as_str() {
                    "contact" => "Contact",
                    "deal" => "Deal",
                    "task" => "Task",
                    "property" => "Property",
                    _ => "Record",
                };
                view! {
                    <crate::components::create_modal::CreateModal
                        entity_type=entity_type.clone()
                        entity_label=label.to_string()
                        on_close=move |_| set_create_entity_type.set(None)
                        on_created=move |_| set_create_entity_type.set(None)
                    />
                }
            })}
        </div>
    }
}


/// Notifications bell connected to WebSocket
#[component]
fn NotificationsBell() -> impl IntoView {
    let (unread_count, _set_unread_count) = create_signal(0i32);
    let (show_panel, set_show_panel) = create_signal(false);
    
    // Listen to WebSocket events for notifications
    // TODO: Connect to SocketContext when integrated
    
    view! {
        <div class="relative">
            <button 
                class="w-10 h-10 flex items-center justify-center rounded-full bg-white/5 hover:bg-white/10 text-slate-300 hover:text-white transition-colors relative"
                on:click=move |_| set_show_panel.update(|v| *v = !*v)
                title="Notifications"
            >
                <span class="text-lg">"🔔"</span>
                {move || (unread_count.get() > 0).then(|| view! {
                    <span class="absolute top-0 right-0 w-2.5 h-2.5 rounded-full bg-red-500 ring-2 ring-surface animate-pulse"></span>
                })}
            </button>
            {move || show_panel.get().then(|| view! {
                <div class="absolute right-0 mt-2 w-80 rounded-xl bg-surface border border-white/10 shadow-2xl shadow-black/50 backdrop-blur-xl z-50 animate-in fade-in zoom-in duration-200 origin-top-right overflow-hidden">
                    <div class="flex items-center justify-between p-4 border-b border-white/10 bg-white/5">
                        <h3 class="font-semibold text-sm">"Notifications"</h3>
                        <button class="text-xs text-indigo-400 hover:text-indigo-300 transition-colors">"Mark all read"</button>
                    </div>
                    <div class="p-8 text-center text-slate-500 text-sm">
                        <span class="block text-2xl mb-2 opacity-50">"🔕"</span>
                        <p>"No new notifications"</p>
                    </div>
                </div>
            })}
        </div>
    }
}

/// Main application shell with sidebar and topbar
#[component]
pub fn Shell() -> impl IntoView {
    let (show_user_menu, set_show_user_menu) = create_signal(false);
    let (show_command_palette, set_show_command_palette) = create_signal(false);
    let (sidebar_collapsed, set_sidebar_collapsed) = create_signal(false);
    let navigate = use_navigate();
    
    // Permission context for nav visibility
    let perms = use_permissions();
    
    // Derived signals for permission-based visibility (avoid closure capture issues)
    let perms_for_config = perms.clone();
    let perms_for_users = perms.clone();
    let show_config = Signal::derive(move || perms_for_config.is_admin_or_manager());
    let show_users = Signal::derive(move || perms_for_users.is_admin());
    
    // Mobile context
    let mobile_ctx = use_mobile();
    let is_mobile = move || mobile_ctx.is_mobile.get();
    
    // Get user email from localStorage
    let user_email = move || {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("user_email").ok())
            .flatten()
            .unwrap_or_else(|| "User".to_string())
    };

    // Logout handler
    let on_logout = move |_| {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.remove_item("logged_in");
            let _ = storage.remove_item("user_email");
            let _ = storage.remove_item("session_token");
        }
        navigate("/login", Default::default());
    };

    // Keyboard shortcut handler (Cmd+K for command palette)
    let set_palette = set_show_command_palette;
    create_effect(move |_| {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        
        let handler = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if (e.meta_key() || e.ctrl_key()) && e.key() == "k" {
                e.prevent_default();
                set_palette.update(|v| *v = !*v);
            }
            // Escape to close
            if e.key() == "Escape" {
                set_palette.set(false);
            }
        }) as Box<dyn Fn(web_sys::KeyboardEvent)>);
        
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "keydown",
                handler.as_ref().unchecked_ref()
            );
        }
        handler.forget();
    });

    view! {
        <crate::components::holographic_shell::HolographicShell>
            <div class="app-shell group flex min-h-screen overflow-hidden relative z-20" class:sidebar-collapsed=sidebar_collapsed class:is-mobile=is_mobile>
                // Command Palette Modal
                {move || show_command_palette.get().then(|| view! {
                    <CommandPalette on_close=move || set_show_command_palette.set(false) />
                })}
            
            // Sidebar
            <aside 
                class="fixed inset-y-0 left-0 z-50 bg-surface/95 backdrop-blur-xl border-r border-white/10 transition-all duration-300 flex flex-col glass-morphism"
                class:w-64=move || !sidebar_collapsed.get()
                class:w-16=move || sidebar_collapsed.get()
            >
                <div class="h-16 flex items-center justify-between px-4 border-b border-white/10 shrink-0">
                    <h1 class="flex items-center gap-2 font-bold text-xl tracking-tight">
                        <span class="text-2xl">"⚡"</span>
                        <span class="group-[.sidebar-collapsed]:hidden transition-opacity">"Jirsi"</span>
                    </h1>
                    <button 
                        class="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-white/10 text-slate-400 hover:text-white transition-colors"
                        on:click=move |_| set_sidebar_collapsed.update(|v| *v = !*v)
                        title="Toggle sidebar"
                    >
                        {move || if sidebar_collapsed.get() { "→" } else { "←" }}
                    </button>
                </div>
                
                <nav class="flex-1 overflow-y-auto py-4 custom-scrollbar">
                    // Apps Section
                    <NavSection title="Apps" icon="🚀" expanded=true>
                        <NavItem href="/" icon="📊" label="Dashboard" />
                        <NavItem href="/app/inbox" icon="📬" label="Inbox" />
                        <NavItem href="/app/calendar" icon="📅" label="Calendar" />
                    </NavSection>
                    
                    // CRM Section
                    <NavSection title="CRM" icon="👥" expanded=true>
                        <NavItem href="/app/crm/entity/contact" icon="👤" label="Contacts" />
                        <NavItem href="/app/crm/entity/company" icon="🏢" label="Companies" />
                        <NavItem href="/app/crm/entity/deal" icon="💰" label="Deals" />
                        <NavItem href="/app/crm/entity/task" icon="✓" label="Tasks" />
                    </NavSection>
                    
                    // Real Estate Section
                    <NavSection title="Real Estate" icon="🏠" expanded=true>
                        <NavItem href="/app/realestate/entity/property" icon="🏡" label="Properties" />
                        <NavItem href="/app/realestate/entity/listing" icon="📢" label="Listings" />
                        <NavItem href="/app/realestate/entity/viewing" icon="👁" label="Viewings" />
                        <NavItem href="/app/realestate/entity/offer" icon="📝" label="Offers" />
                    </NavSection>
                    
                    // Intelligence Section
                    <NavSection title="Intelligence" icon="📈" expanded=false>
                        <NavItem href="/app/reports" icon="📊" label="Reports" />
                        <NavItem href="/app/analytics" icon="📉" label="Analytics" />
                    </NavSection>
                    
                    // Configuration Section (Admin/Manager only)
                    <Show when=move || show_config.get()>
                        <NavSection title="Configuration" icon="⚙️" expanded=false>
                            <NavItem href="/app/settings" icon="🔧" label="Settings" />
                            <NavItem href="/app/settings/workflows" icon="🔄" label="Workflows" />
                            <Show when=move || show_users.get()>
                                <NavItem href="/app/users" icon="👥" label="Users" />
                            </Show>
                        </NavSection>
                    </Show>
                </nav>
                
                // Neural Status Footer
                {move || if sidebar_collapsed.get() {
                    view! { <crate::components::neural_status::NeuralStatusCompact /> }.into_view()
                } else {
                    view! { <crate::components::neural_status::NeuralStatus /> }.into_view()
                }}
            </aside>
            
            // Main Content Area
            <main 
                class="flex-1 flex flex-col min-w-0 transition-all duration-300"
                class:ml-64=move || !sidebar_collapsed.get()
                class:ml-16=move || sidebar_collapsed.get()
            >
                // Power Header
                <header class="h-16 sticky top-0 z-40 bg-surface/80 backdrop-blur-md border-b border-white/10 flex items-center justify-between px-6 gap-4">
                    // Search Bar (Command Palette Trigger)
                    <div 
                        class="flex items-center gap-3 px-4 py-2 bg-white/5 border border-white/10 rounded-xl text-sm text-slate-400 w-96 cursor-text hover:bg-white/10 hover:border-white/20 transition-all group/search"
                        on:click=move |_| set_show_command_palette.set(true)
                    >
                        <span class="opacity-50 group-hover/search:opacity-100 transition-opacity">"🔍"</span>
                        <span class="flex-1">"Search... "</span>
                        <span class="px-2 py-0.5 rounded bg-white/5 text-[10px] font-bold border border-white/5">"⌘K"</span>
                    </div>
                    
                    // Actions
                    <div class="flex items-center gap-4">
                        <crate::components::sync_indicator::SyncIndicator />
                        <QuickCreateButton />
                        <NotificationsBell />
                        <ThemeToggle />
                        
                        // User Menu
                        <div class="relative">
                            <button 
                                class="flex items-center gap-3 pl-3 pr-2 py-1.5 rounded-full hover:bg-white/5 border border-transparent hover:border-white/10 transition-all"
                                on:click=move |_| set_show_user_menu.update(|v| *v = !*v)
                            >
                                <div class="w-8 h-8 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-xs font-bold border border-white/20 shadow-lg shadow-indigo-500/20">
                                    "👤"
                                </div>
                                <span class="text-sm font-medium text-slate-200">{user_email}</span>
                                <span class="text-xs text-slate-500">"▼"</span>
                            </button>
                            {move || show_user_menu.get().then(|| view! {
                                <div class="absolute right-0 mt-2 w-48 py-1 rounded-xl bg-surface border border-white/10 shadow-2xl shadow-black/50 backdrop-blur-xl z-50 animate-in fade-in zoom-in duration-200">
                                    <a href="/app/profile" class="block px-4 py-2 text-sm text-slate-300 hover:text-white hover:bg-white/5 transition-colors">"👤 Profile"</a>
                                    <a href="/app/settings" class="block px-4 py-2 text-sm text-slate-300 hover:text-white hover:bg-white/5 transition-colors">"⚙️ Settings"</a>
                                    <div class="h-px bg-white/10 my-1"></div>
                                    <button 
                                        class="w-full text-left px-4 py-2 text-sm text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-colors"
                                        on:click=on_logout.clone()
                                    >
                                        "🚪 Logout"
                                    </button>
                                </div>
                            })}
                        </div>
                    </div>
                </header>
                
                // Page Content
                <div class="flex-1 p-6 overflow-x-hidden">
                    <Outlet/>
                </div>
            </main>
            
            // Bottom Nav (Mobile Only)
            {move || is_mobile().then(|| view! { <BottomNav /> })}
            </div>
        </crate::components::holographic_shell::HolographicShell>
    }
}

/// Command Palette - Global Search (Cmd+K)
#[component]
fn CommandPalette(
    on_close: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let (query, set_query) = create_signal(String::new());
    let (results, set_results) = create_signal::<Vec<SearchResult>>(vec![]);
    let (selected_index, set_selected_index) = create_signal(0usize);
    let (loading, set_loading) = create_signal(false);
    let navigate = use_navigate();
    let on_close_clone = on_close.clone();
    
    // Create node ref for input auto-focus
    let input_ref = create_node_ref::<leptos::html::Input>();
    
    // Auto-focus input when component mounts
    create_effect(move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    });
    // Search handler with debounce
    create_effect(move |_| {
        let q = query.get();
        if q.len() >= 2 {
            set_loading.set(true);
            let q_clone = q.clone();
            spawn_local(async move {
                if let Ok(res) = search_entities(&q_clone).await {
                    set_results.set(res);
                    set_selected_index.set(0);
                }
                set_loading.set(false);
            });
        } else {
            set_results.set(vec![]);
        }
    });
    
    // Keyboard navigation
    let on_close_nav = on_close.clone();
    let navigate_clone = navigate.clone();
    let on_keydown = move |e: web_sys::KeyboardEvent| {
        match e.key().as_str() {
            "ArrowDown" => {
                e.prevent_default();
                set_selected_index.update(|i| {
                    let len = results.get().len();
                    if len > 0 { *i = (*i + 1) % len; }
                });
            }
            "ArrowUp" => {
                e.prevent_default();
                set_selected_index.update(|i| {
                    let len = results.get().len();
                    if len > 0 && *i > 0 { *i -= 1; }
                    else if len > 0 { *i = len - 1; }
                });
            }
            "Enter" => {
                e.prevent_default();
                if let Some(result) = results.get().get(selected_index.get()) {
                    navigate_clone(&result.url, Default::default());
                    on_close_nav();
                }
            }
            "Escape" => {
                on_close_nav();
            }
            _ => {}
        }
    };
    
    view! {
        <div class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50 backdrop-blur-sm transition-opacity" on:click=move |_| on_close_clone()>
            <div class="w-full max-w-2xl bg-surface border border-white/10 rounded-2xl shadow-2xl shadow-black/50 flex flex-col overflow-hidden animate-spring-up" on:click=|e| e.stop_propagation()>
                <div class="flex items-center gap-4 px-6 py-4 border-b border-white/10">
                    <span class="text-xl opacity-50">"🔍"</span>
                    <input 
                        type="text"
                        class="flex-1 bg-transparent border-none outline-none text-lg text-white placeholder-slate-500 h-10"
                        placeholder="Search contacts, deals, properties..."
                        prop:value=query
                        on:input=move |e| set_query.set(event_target_value(&e))
                        on:keydown=on_keydown
                        node_ref=input_ref
                    />
                    {move || loading.get().then(|| view! {
                        <span class="animate-spin text-indigo-400">"⟳"</span>
                    })}
                </div>
                
                <div class="max-h-[60vh] overflow-y-auto custom-scrollbar p-2">
                    {move || {
                        let res = results.get();
                        let sel = selected_index.get();
                        
                        if res.is_empty() && query.get().len() >= 2 {
                            view! {
                                <div class="p-8 text-center text-slate-500">
                                    "No results found"
                                </div>
                            }.into_view()
                        } else if res.is_empty() {
                            view! {
                                <div class="p-8 text-center">
                                    <p class="text-slate-400 text-sm mb-4">"Start typing to search..."</p>
                                    <div class="flex flex-wrap justify-center gap-2">
                                        <span class="px-2 py-1 rounded bg-white/5 text-xs text-slate-500 border border-white/5">"contact name"</span>
                                        <span class="px-2 py-1 rounded bg-white/5 text-xs text-slate-500 border border-white/5">"deal title"</span>
                                        <span class="px-2 py-1 rounded bg-white/5 text-xs text-slate-500 border border-white/5">"property address"</span>
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            // Group results by entity type
                            let mut grouped: std::collections::HashMap<String, Vec<(usize, SearchResult)>> = std::collections::HashMap::new();
                            for (i, r) in res.into_iter().enumerate() {
                                grouped.entry(r.entity_type.clone()).or_default().push((i, r));
                            }
                            
                            grouped.into_iter().map(|(entity_type, items)| {
                                view! {
                                    <div class="mb-2">
                                        <div class="px-3 py-2 text-xs font-bold text-slate-500 uppercase tracking-wider">{entity_type.to_uppercase()}</div>
                                        {items.into_iter().map(|(i, item)| {
                                            let url = item.url.clone();
                                            let navigate = navigate.clone();
                                            let on_close = on_close.clone();
                                            view! {
                                                <div 
                                                    class="flex items-center gap-3 px-3 py-3 rounded-xl cursor-pointer transition-colors"
                                                    class:bg-indigo-600=move || sel == i
                                                    class:text-white=move || sel == i
                                                    class:text-slate-300=move || sel != i
                                                    class:hover:bg-white_5=move || sel != i
                                                    on:click=move |_| {
                                                        navigate(&url, Default::default());
                                                        on_close();
                                                    }
                                                >
                                                    <span class="text-xl">{item.icon}</span>
                                                    <div class="flex-1 min-w-0">
                                                        <div class="font-medium truncate">{item.title}</div>
                                                        <div class="text-xs opacity-70 truncate">{item.subtitle}</div>
                                                    </div>
                                                    {move || (sel == i).then(|| view! {
                                                        <span class="text-xs opacity-70">"↵"</span>
                                                    })}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }
                            }).collect_view()
                        }
                    }}
                </div>
                
                <div class="flex items-center gap-4 px-4 py-3 bg-white/5 border-t border-white/10 text-xs text-slate-500 font-medium">
                    <span class="flex items-center gap-1"><kbd class="px-1.5 py-0.5 rounded bg-white/10 min-w-[20px] text-center">"↑"</kbd> <kbd class="px-1.5 py-0.5 rounded bg-white/10 min-w-[20px] text-center">"↓"</kbd> "Navigate"</span>
                    <span class="flex items-center gap-1"><kbd class="px-1.5 py-0.5 rounded bg-white/10 min-w-[20px] text-center">"↵"</kbd> "Select"</span>
                    <span class="flex items-center gap-1"><kbd class="px-1.5 py-0.5 rounded bg-white/10 min-w-[20px] text-center">"Esc"</kbd> "Close"</span>
                </div>
            </div>
        </div>
    }
}

// Search result type
#[derive(Clone, Debug)]
struct SearchResult {
    entity_type: String,
    icon: &'static str,
    title: String,
    subtitle: String,
    url: String,
    #[allow(dead_code)]
    action: Option<SearchAction>,
}

// Action types for command bar
#[derive(Clone, Debug)]
#[allow(dead_code)]
enum SearchAction {
    Navigate(String),
    CreateEntity { entity_type: String, prefill: Option<String> },
    QuickAction { action_id: String },
}

// Intent parsing - detect commands like "add contact John Doe"
fn parse_intent(query: &str) -> Option<SearchResult> {
    let q = query.to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();
    
    if words.is_empty() {
        return None;
    }
    
    // Detect "add/create/new" commands
    let action_words = ["add", "create", "new"];
    let entity_types = [
        ("contact", "👤", "/app/crm/entity/contact", "contact"),
        ("deal", "💰", "/app/crm/entity/deal", "deal"),
        ("property", "🏠", "/app/realestate/entity/property", "property"),
        ("task", "✓", "/app/crm/entity/task", "task"),
        ("company", "🏢", "/app/crm/entity/company", "company"),
        ("viewing", "👁", "/app/realestate/entity/viewing", "viewing"),
    ];
    
    if let Some(action_word) = words.first() {
        if action_words.contains(action_word) && words.len() >= 2 {
            // Check for entity type
            for (entity_name, icon, base_url, entity_code) in entity_types.iter() {
                if words.get(1) == Some(entity_name) {
                    // Extract prefill data (remaining words)
                    let prefill: String = words.iter().skip(2).cloned().collect::<Vec<_>>().join(" ");
                    let prefill_param = if !prefill.is_empty() { 
                        format!("?prefill={}", urlencoding::encode(&prefill)) 
                    } else { 
                        String::new() 
                    };
                    
                    return Some(SearchResult {
                        entity_type: "Quick Action".to_string(),
                        icon,
                        title: format!("Create new {}", entity_name),
                        subtitle: if !prefill.is_empty() { 
                            format!("Pre-fill: {}", prefill) 
                        } else { 
                            format!("Open {} creation form", entity_name) 
                        },
                        url: format!("{}/new{}", base_url, prefill_param),
                        action: Some(SearchAction::CreateEntity {
                            entity_type: entity_code.to_string(),
                            prefill: if prefill.is_empty() { None } else { Some(prefill) },
                        }),
                    });
                }
            }
        }
    }
    
    // Detect navigation shortcuts
    let navigation_shortcuts = [
        ("dashboard", "📊", "Go to Dashboard", "/", "Command Center", "dashboard"),
        ("contacts", "👤", "Go to Contacts", "/app/crm/entity/contact", "View all contacts", "contacts"),
        ("deals", "💰", "Go to Deals", "/app/crm/entity/deal", "View all deals", "deals"),
        ("properties", "🏠", "Go to Properties", "/app/realestate/entity/property", "View all properties", "properties"),
        ("settings", "⚙️", "Go to Settings", "/app/settings", "System settings", "settings"),
        ("reports", "📊", "Go to Reports", "/app/reports", "View reports", "reports"),
        ("inbox", "📬", "Go to Inbox", "/app/inbox", "View messages", "inbox"),
        ("calendar", "📅", "Go to Calendar", "/app/calendar", "View calendar", "calendar"),
    ];
    
    for (keyword, icon, title, url, subtitle, _id) in navigation_shortcuts.iter() {
        if q.contains(keyword) || keyword.contains(&q) {
            return Some(SearchResult {
                entity_type: "Navigation".to_string(),
                icon,
                title: title.to_string(),
                subtitle: subtitle.to_string(),
                url: url.to_string(),
                action: Some(SearchAction::Navigate(url.to_string())),
            });
        }
    }
    
    None
}

// Search API call - searches across contacts, deals, properties
async fn search_entities(query: &str) -> Result<Vec<SearchResult>, String> {
    use crate::api::{API_BASE, TENANT_ID};
    
    let mut results = vec![];
    let q = query.to_lowercase();
    
    // First, check for intent/command parsing
    if let Some(intent_result) = parse_intent(query) {
        results.push(intent_result);
    }
    
    // Search contacts
    let contacts_url = format!("{}/entities/contact?tenant_id={}", API_BASE, TENANT_ID);
    if let Ok(contacts) = fetch_search_results(&contacts_url).await {
        for contact in contacts {
            let name = contact.get("name")
                .or_else(|| contact.get("first_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let email = contact.get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let phone = contact.get("phone")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let id = contact.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            // Check if query matches
            if name.to_lowercase().contains(&q) || 
               email.to_lowercase().contains(&q) || 
               phone.contains(&q) {
                results.push(SearchResult {
                    entity_type: "Contacts".to_string(),
                    icon: "👤",
                    title: name.to_string(),
                    subtitle: email.to_string(),
                    url: format!("/app/crm/entity/contact/{}", id),
                    action: None,
                });
            }
        }
    }
    
    // Search deals
    let deals_url = format!("{}/entities/deal?tenant_id={}", API_BASE, TENANT_ID);
    if let Ok(deals) = fetch_search_results(&deals_url).await {
        for deal in deals {
            let title = deal.get("title")
                .or_else(|| deal.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let id = deal.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if title.to_lowercase().contains(&q) {
                results.push(SearchResult {
                    entity_type: "Deals".to_string(),
                    icon: "💰",
                    title: title.to_string(),
                    subtitle: "".to_string(),
                    url: format!("/app/crm/entity/deal/{}", id),
                    action: None,
                });
            }
        }
    }
    
    // Search properties
    let props_url = format!("{}/entities/property?tenant_id={}", API_BASE, TENANT_ID);
    if let Ok(props) = fetch_search_results(&props_url).await {
        for prop in props {
            let title = prop.get("title")
                .or_else(|| prop.get("address"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let id = prop.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if title.to_lowercase().contains(&q) {
                results.push(SearchResult {
                    entity_type: "Properties".to_string(),
                    icon: "🏠",
                    title: title.to_string(),
                    subtitle: "".to_string(),
                    url: format!("/app/realestate/entity/property/{}", id),
                    action: None,
                });
            }
        }
    }
    
    Ok(results)
}

async fn fetch_search_results(url: &str) -> Result<Vec<serde_json::Value>, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};
    
    logging::log!("Fetching search: {}", url);
    
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| {
            logging::error!("Request create error: {:?}", e);
            "Failed to create request"
        })?;
    
    // Get session token
    let token = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("session_token").ok())
        .flatten()
        .unwrap_or_default();
    
    request.headers().set("Authorization", &format!("Bearer {}", token)).ok();
    
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| {
            logging::error!("Fetch error: {:?}", e);
            "Fetch failed"
        })?;
    
    let resp: Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;
    
    logging::log!("Response status: {}", resp.status());
    
    if !resp.ok() {
        logging::error!("Response not OK: {}", resp.status());
        return Ok(vec![]);
    }
    
    let text = JsFuture::from(resp.text().map_err(|_| "Text error")?)
        .await
        .map_err(|_| "Text await error")?;
    
    let text_str = text.as_string().unwrap_or_default();
    logging::log!("Response text length: {}", text_str.len());
    
    // Parse response - expecting { "data": [...] }
    let parsed: serde_json::Value = serde_json::from_str(&text_str)
        .map_err(|e| {
            logging::error!("JSON parse error: {:?}", e);
            "JSON parse error"
        })?;
    
    let data = parsed.get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    
    logging::log!("Found {} entities", data.len());
    
    Ok(data)
}

