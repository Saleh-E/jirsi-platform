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
        <div class="nav-section" class:collapsed=move || !is_expanded.get()>
            <button 
                class="nav-section-header"
                on:click=move |_| set_expanded.update(|v| *v = !*v)
            >
                <span class="section-icon">{icon}</span>
                <span class="section-title">{title}</span>
                <span class="section-chevron">
                    {move || if is_expanded.get() { "▼" } else { "▶" }}
                </span>
            </button>
            <div class="nav-section-items" class:hidden=move || !is_expanded.get()>
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
            class="nav-item"
            class:active=is_active
        >
            <span class="nav-icon">{icon}</span>
            <span class="nav-label">{label}</span>
            {badge.map(|b| view! {
                <span class="nav-badge">{move || b.get()}</span>
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
        <div class="quick-create-container">
            <button 
                class="btn-quick-create"
                on:click=move |_| set_show_menu.update(|v| *v = !*v)
                title="Quick Create"
            >
                <span class="icon">"+"</span>
            </button>
            {move || show_menu.get().then(|| {
                view! {
                    <div class="quick-create-menu">
                        <button class="menu-item" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("contact".to_string()));
                        }>
                            <span>"👤"</span> "New Contact"
                        </button>
                        <button class="menu-item" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("deal".to_string()));
                        }>
                            <span>"💰"</span> "New Deal"
                        </button>
                        <button class="menu-item" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("task".to_string()));
                        }>
                            <span>"✓"</span> "New Task"
                        </button>
                        <button class="menu-item" on:click=move |_| {
                            set_show_menu.set(false);
                            set_create_entity_type.set(Some("property".to_string()));
                        }>
                            <span>"🏠"</span> "New Property"
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
        <div class="notifications-container">
            <button 
                class="btn-notifications"
                on:click=move |_| set_show_panel.update(|v| *v = !*v)
                title="Notifications"
            >
                <span class="icon">"🔔"</span>
                {move || (unread_count.get() > 0).then(|| view! {
                    <span class="notification-badge">{unread_count}</span>
                })}
            </button>
            {move || show_panel.get().then(|| view! {
                <div class="notifications-panel">
                    <div class="panel-header">
                        <h3>"Notifications"</h3>
                        <button class="btn-mark-read">"Mark all read"</button>
                    </div>
                    <div class="panel-body">
                        <p class="empty-state">"No new notifications"</p>
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
        <div class="app-shell" class:sidebar-collapsed=sidebar_collapsed class:is-mobile=is_mobile>
            // Command Palette Modal
            {move || show_command_palette.get().then(|| view! {
                <CommandPalette on_close=move || set_show_command_palette.set(false) />
            })}
            
            // Sidebar
            <aside class="sidebar">
                <div class="sidebar-header">
                    <h1 class="logo">
                        <span class="logo-icon">"⚡"</span>
                        <span class="logo-text" class:hidden=sidebar_collapsed>"Jirsi"</span>
                    </h1>
                    <button 
                        class="btn-collapse"
                        on:click=move |_| set_sidebar_collapsed.update(|v| *v = !*v)
                        title="Toggle sidebar"
                    >
                        {move || if sidebar_collapsed.get() { "→" } else { "←" }}
                    </button>
                </div>
                
                <nav class="sidebar-nav">
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
            </aside>
            
            // Main Content Area
            <main class="main-content">
                // Power Header
                <header class="topbar">
                    // Search Bar (Command Palette Trigger)
                    <div 
                        class="search-bar"
                        on:click=move |_| set_show_command_palette.set(true)
                    >
                        <span class="search-icon">"🔍"</span>
                        <span class="search-placeholder">"Search... "</span>
                        <span class="search-shortcut">"⌘K"</span>
                    </div>
                    
                    // Actions
                    <div class="topbar-actions">
                        <crate::components::sync_indicator::SyncIndicator />
                        <QuickCreateButton />
                        <NotificationsBell />
                        <ThemeToggle />
                        
                        // User Menu
                        <div class="user-menu-container">
                            <button 
                                class="user-btn" 
                                on:click=move |_| set_show_user_menu.update(|v| *v = !*v)
                            >
                                <span class="user-avatar">"👤"</span>
                                <span class="user-email">{user_email}</span>
                                <span class="arrow">"▼"</span>
                            </button>
                            {move || show_user_menu.get().then(|| view! {
                                <div class="user-dropdown">
                                    <a href="/app/profile" class="dropdown-item">"👤 Profile"</a>
                                    <a href="/app/settings" class="dropdown-item">"⚙️ Settings"</a>
                                    <hr/>
                                    <button class="dropdown-item logout" on:click=on_logout.clone()>
                                        "🚪 Logout"
                                    </button>
                                </div>
                            })}
                        </div>
                    </div>
                </header>
                
                // Page Content
                <div class="content">
                    <Outlet/>
                </div>
            </main>
            
            // Bottom Nav (Mobile Only)
            {move || is_mobile().then(|| view! { <BottomNav /> })}
        </div>
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
        <div class="command-palette-overlay" on:click=move |_| on_close_clone()>
            <div class="command-palette" on:click=|e| e.stop_propagation()>
                <div class="palette-input-wrapper">
                    <span class="search-icon">"🔍"</span>
                    <input 
                        type="text"
                        class="palette-input"
                        placeholder="Search contacts, deals, properties..."
                        prop:value=query
                        on:input=move |e| set_query.set(event_target_value(&e))
                        on:keydown=on_keydown
                        node_ref=input_ref
                    />
                    {move || loading.get().then(|| view! {
                        <span class="loading-spinner">"⟳"</span>
                    })}
                </div>
                
                <div class="palette-results">
                    {move || {
                        let res = results.get();
                        let sel = selected_index.get();
                        
                        if res.is_empty() && query.get().len() >= 2 {
                            view! {
                                <div class="no-results">
                                    "No results found"
                                </div>
                            }.into_view()
                        } else if res.is_empty() {
                            view! {
                                <div class="search-hint">
                                    <p>"Start typing to search..."</p>
                                    <div class="quick-actions">
                                        <span class="hint">"Try: contact name, deal title, property address"</span>
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
                                    <div class="result-group">
                                        <div class="group-header">{entity_type.to_uppercase()}</div>
                                        {items.into_iter().map(|(i, item)| {
                                            let url = item.url.clone();
                                            let navigate = navigate.clone();
                                            let on_close = on_close.clone();
                                            view! {
                                                <div 
                                                    class="result-item"
                                                    class:selected=move || sel == i
                                                    on:click=move |_| {
                                                        navigate(&url, Default::default());
                                                        on_close();
                                                    }
                                                >
                                                    <span class="result-icon">{item.icon}</span>
                                                    <span class="result-title">{item.title}</span>
                                                    <span class="result-subtitle">{item.subtitle}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }
                            }).collect_view()
                        }
                    }}
                </div>
                
                <div class="palette-footer">
                    <span>"↑↓ Navigate"</span>
                    <span>"↵ Select"</span>
                    <span>"Esc Close"</span>
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
}

// Search API call - searches across contacts, deals, properties
async fn search_entities(query: &str) -> Result<Vec<SearchResult>, String> {
    use crate::api::{API_BASE, TENANT_ID};
    
    let mut results = vec![];
    let q = query.to_lowercase();
    
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
                    icon: "�",
                    title: title.to_string(),
                    subtitle: "".to_string(),
                    url: format!("/app/crm/entity/deal/{}", id),
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

