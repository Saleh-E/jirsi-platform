//! Command Palette Component
//!
//! A Ctrl+K activated command palette for global search and navigation.
//! Features: fuzzy search with typo tolerance, keyboard navigation, categories.

use leptos::*;
use leptos_router::use_navigate;
use wasm_bindgen::prelude::*;

/// Command item for the palette
#[derive(Clone, Debug)]
pub struct CommandItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: String,  // navigation, action, recent
    pub icon: String,
    pub action: CommandAction,
    pub keywords: Vec<String>,  // Additional search keywords
}

/// Action to perform when command is selected
#[derive(Clone, Debug)]
pub enum CommandAction {
    Navigate(String),
    Callback(String),  // Store callback name, handle in component
}

/// Command Palette context for external control
#[derive(Clone)]
pub struct CommandPaletteContext {
    pub is_open: RwSignal<bool>,
}

/// Hook to get command palette context
pub fn use_command_palette() -> Option<CommandPaletteContext> {
    use_context::<CommandPaletteContext>()
}

/// Fuzzy match score for search
fn fuzzy_score(query: &str, text: &str, keywords: &[String]) -> Option<i32> {
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    
    // Exact match = highest score
    if t.starts_with(&q) {
        return Some(1000 - q.len() as i32);
    }
    
    // Contains match
    if t.contains(&q) {
        return Some(500 - q.len() as i32);
    }
    
    // Keyword match
    for kw in keywords {
        if kw.to_lowercase().contains(&q) {
            return Some(300);
        }
    }
    
    // Fuzzy match - check if chars appear in order
    let mut score = 0;
    let mut query_idx = 0;
    let query_chars: Vec<char> = q.chars().collect();
    
    for (i, c) in t.char_indices() {
        if query_idx < query_chars.len() && c == query_chars[query_idx] {
            // Bonus for matching at word start
            if i == 0 || t.chars().nth(i - 1).map(|p| p == ' ').unwrap_or(false) {
                score += 50;
            } else {
                score += 10;
            }
            query_idx += 1;
        }
    }
    
    // Only return score if all query chars were found
    if query_idx == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

/// Command Palette Provider - wrap your app to enable Ctrl+K
#[component]
pub fn CommandPaletteProvider(children: Children) -> impl IntoView {
    let is_open = create_rw_signal(false);
    
    // Provide context
    provide_context(CommandPaletteContext { is_open });
    
    // Global keyboard listener for Ctrl+K
    create_effect(move |_| {
        let closure = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            if (e.ctrl_key() || e.meta_key()) && e.key() == "k" {
                e.prevent_default();
                is_open.update(|v| *v = !*v);
            }
            // Close on Escape
            if e.key() == "Escape" {
                is_open.set(false);
            }
        });
        
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "keydown",
                closure.as_ref().unchecked_ref(),
            );
        }
        
        // Keep closure alive
        closure.forget();
    });
    
    view! {
        {children()}
        <CommandPalette is_open=is_open />
    }
}

/// Command Palette Modal
#[component]
fn CommandPalette(is_open: RwSignal<bool>) -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (selected_index, set_selected_index) = create_signal(0usize);
    let navigate = use_navigate();
    let input_ref = create_node_ref::<leptos::html::Input>();
    
    // Auto-focus input when palette opens
    create_effect(move |_| {
        if is_open.get() {
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    });
    
    // Store commands statically with keywords for better search
    let commands: Vec<CommandItem> = vec![
        // Navigation
        CommandItem {
            id: "nav-dashboard".to_string(),
            title: "Go to Dashboard".to_string(),
            subtitle: Some("Overview & analytics".to_string()),
            category: "Navigation".to_string(),
            icon: "📊".to_string(),
            action: CommandAction::Navigate("/".to_string()),
            keywords: vec!["home".to_string(), "main".to_string(), "stats".to_string()],
        },
        CommandItem {
            id: "nav-contacts".to_string(),
            title: "Go to Contacts".to_string(),
            subtitle: Some("View all contacts".to_string()),
            category: "Navigation".to_string(),
            icon: "👤".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/contact".to_string()),
            keywords: vec!["people".to_string(), "customers".to_string(), "leads".to_string()],
        },
        CommandItem {
            id: "nav-companies".to_string(),
            title: "Go to Companies".to_string(),
            subtitle: Some("View all companies".to_string()),
            category: "Navigation".to_string(),
            icon: "🏢".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/company".to_string()),
            keywords: vec!["organizations".to_string(), "businesses".to_string()],
        },
        CommandItem {
            id: "nav-deals".to_string(),
            title: "Go to Deals".to_string(),
            subtitle: Some("View all deals".to_string()),
            category: "Navigation".to_string(),
            icon: "💰".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/deal".to_string()),
            keywords: vec!["sales".to_string(), "pipeline".to_string(), "opportunities".to_string()],
        },
        CommandItem {
            id: "nav-properties".to_string(),
            title: "Go to Properties".to_string(),
            subtitle: Some("View all properties".to_string()),
            category: "Navigation".to_string(),
            icon: "🏠".to_string(),
            action: CommandAction::Navigate("/app/realestate/entity/property".to_string()),
            keywords: vec!["real estate".to_string(), "listings".to_string(), "homes".to_string()],
        },
        CommandItem {
            id: "nav-tasks".to_string(),
            title: "Go to Tasks".to_string(),
            subtitle: Some("View all tasks".to_string()),
            category: "Navigation".to_string(),
            icon: "✓".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/task".to_string()),
            keywords: vec!["todo".to_string(), "checklist".to_string(), "action items".to_string()],
        },
        CommandItem {
            id: "nav-reports".to_string(),
            title: "Go to Reports".to_string(),
            subtitle: Some("View analytics".to_string()),
            category: "Navigation".to_string(),
            icon: "📈".to_string(),
            action: CommandAction::Navigate("/app/reports".to_string()),
            keywords: vec!["analytics".to_string(), "metrics".to_string(), "data".to_string()],
        },
        CommandItem {
            id: "nav-inbox".to_string(),
            title: "Go to Inbox".to_string(),
            subtitle: Some("View messages".to_string()),
            category: "Navigation".to_string(),
            icon: "📧".to_string(),
            action: CommandAction::Navigate("/app/inbox".to_string()),
            keywords: vec!["email".to_string(), "messages".to_string(), "communication".to_string()],
        },
        CommandItem {
            id: "nav-calendar".to_string(),
            title: "Go to Calendar".to_string(),
            subtitle: Some("View schedule".to_string()),
            category: "Navigation".to_string(),
            icon: "📅".to_string(),
            action: CommandAction::Navigate("/app/calendar".to_string()),
            keywords: vec!["schedule".to_string(), "events".to_string(), "meetings".to_string()],
        },
        CommandItem {
            id: "nav-settings".to_string(),
            title: "Go to Settings".to_string(),
            subtitle: Some("Configure your workspace".to_string()),
            category: "Navigation".to_string(),
            icon: "⚙️".to_string(),
            action: CommandAction::Navigate("/app/settings".to_string()),
            keywords: vec!["preferences".to_string(), "config".to_string()],
        },
        // Actions
        CommandItem {
            id: "action-new-contact".to_string(),
            title: "Create New Contact".to_string(),
            subtitle: Some("Add a new contact".to_string()),
            category: "Quick Actions".to_string(),
            icon: "➕".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/contact?create=true".to_string()),
            keywords: vec!["add".to_string(), "new".to_string(), "person".to_string()],
        },
        CommandItem {
            id: "action-new-deal".to_string(),
            title: "Create New Deal".to_string(),
            subtitle: Some("Start a new deal".to_string()),
            category: "Quick Actions".to_string(),
            icon: "➕".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/deal?create=true".to_string()),
            keywords: vec!["add".to_string(), "new".to_string(), "sale".to_string()],
        },
        CommandItem {
            id: "action-new-task".to_string(),
            title: "Create New Task".to_string(),
            subtitle: Some("Add a new task".to_string()),
            category: "Quick Actions".to_string(),
            icon: "➕".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/task?create=true".to_string()),
            keywords: vec!["add".to_string(), "new".to_string(), "todo".to_string()],
        },
        CommandItem {
            id: "action-new-property".to_string(),
            title: "Create New Property".to_string(),
            subtitle: Some("Add a new property".to_string()),
            category: "Quick Actions".to_string(),
            icon: "➕".to_string(),
            action: CommandAction::Navigate("/app/realestate/entity/property?create=true".to_string()),
            keywords: vec!["add".to_string(), "new".to_string(), "listing".to_string()],
        },
        // Workflows
        CommandItem {
            id: "nav-workflows".to_string(),
            title: "Manage Workflows".to_string(),
            subtitle: Some("Automation & workflow builder".to_string()),
            category: "Configuration".to_string(),
            icon: "🔄".to_string(),
            action: CommandAction::Navigate("/app/settings/workflows".to_string()),
            keywords: vec!["automation".to_string(), "flow".to_string()],
        },
        CommandItem {
            id: "nav-users".to_string(),
            title: "Manage Users".to_string(),
            subtitle: Some("Team & permissions".to_string()),
            category: "Configuration".to_string(),
            icon: "👥".to_string(),
            action: CommandAction::Navigate("/app/users".to_string()),
            keywords: vec!["team".to_string(), "permissions".to_string(), "roles".to_string()],
        },
    ];
    
    // Store commands for reactive access
    let commands_store = store_value(commands);
    
    // Filter commands based on search with fuzzy scoring
    let filtered_commands = move || {
        let query = search_query.get();
        let cmds = commands_store.get_value();
        
        if query.is_empty() {
            return cmds;
        }
        
        let mut scored: Vec<(i32, CommandItem)> = cmds.into_iter()
            .filter_map(|cmd| {
                fuzzy_score(&query, &cmd.title, &cmd.keywords).map(|s| (s, cmd))
            })
            .collect();
        
        // Sort by score descending
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, cmd)| cmd).collect()
    };
    
    // Signal to trigger navigation (workaround for closure issues)
    let (navigate_to, set_navigate_to) = create_signal::<Option<String>>(None);
    
    // Effect to handle navigation
    create_effect(move |_| {
        if let Some(path) = navigate_to.get() {
            navigate(&path, Default::default());
            set_navigate_to.set(None);
        }
    });
    
    // Reset selection when search changes
    create_effect(move |_| {
        let _ = search_query.get();
        set_selected_index.set(0);
    });
    
    // Close and clear on close
    let close = move |_| {
        is_open.set(false);
        set_search_query.set(String::new());
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="command-palette-backdrop" on:click=close></div>
            <div class="command-palette">
                <div class="command-search">
                    <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="8"></circle>
                        <path d="m21 21-4.35-4.35"></path>
                    </svg>
                    <input 
                        type="text"
                        class="command-input"
                        placeholder="What do you need?"
                        node_ref=input_ref
                        on:input=move |e| set_search_query.set(event_target_value(&e))
                        on:keydown=move |e: web_sys::KeyboardEvent| {
                            let cmds = filtered_commands();
                            let max_index = cmds.len().saturating_sub(1);
                            
                            match e.key().as_str() {
                                "ArrowDown" => {
                                    e.prevent_default();
                                    set_selected_index.update(|i| {
                                        if *i < max_index { *i += 1; }
                                    });
                                }
                                "ArrowUp" => {
                                    e.prevent_default();
                                    set_selected_index.update(|i| {
                                        if *i > 0 { *i -= 1; }
                                    });
                                }
                                "Enter" => {
                                    e.prevent_default();
                                    if let Some(cmd) = cmds.get(selected_index.get()) {
                                        if let CommandAction::Navigate(ref path) = cmd.action {
                                            set_navigate_to.set(Some(path.clone()));
                                        }
                                        is_open.set(false);
                                        set_search_query.set(String::new());
                                    }
                                }
                                _ => {}
                            }
                        }
                        prop:value=search_query
                    />
                    <span class="command-shortcut">"ESC"</span>
                </div>
                
                <div class="command-list">
                    {move || {
                        let cmds = filtered_commands();
                        let selected = selected_index.get();
                        
                        if cmds.is_empty() {
                            view! {
                                <div class="command-empty">
                                    <span class="empty-icon">"🔍"</span>
                                    <span>"No results found"</span>
                                </div>
                            }.into_view()
                        } else {
                            // Group by category
                            let mut categories: Vec<(String, Vec<(usize, CommandItem)>)> = vec![];
                            let mut current_cat = String::new();
                            
                            for (idx, cmd) in cmds.iter().enumerate() {
                                if cmd.category != current_cat {
                                    current_cat = cmd.category.clone();
                                    categories.push((current_cat.clone(), vec![]));
                                }
                                if let Some((_, items)) = categories.last_mut() {
                                    items.push((idx, cmd.clone()));
                                }
                            }
                            
                            categories.into_iter().map(|(cat, items)| {
                                view! {
                                    <div class="command-category">
                                        <div class="category-title">{cat}</div>
                                        {items.into_iter().map(|(idx, cmd)| {
                                            let is_selected = idx == selected;
                                            let item_class = if is_selected { "command-item selected" } else { "command-item" };
                                            let path = match &cmd.action {
                                                CommandAction::Navigate(p) => p.clone(),
                                                _ => String::new(),
                                            };
                                            
                                            view! {
                                                <div 
                                                    class=item_class
                                                    on:click=move |_| {
                                                        set_navigate_to.set(Some(path.clone()));
                                                        is_open.set(false);
                                                        set_search_query.set(String::new());
                                                    }
                                                    on:mouseenter=move |_| {
                                                        set_selected_index.set(idx);
                                                    }
                                                >
                                                    <span class="command-icon">{cmd.icon.clone()}</span>
                                                    <div class="command-text">
                                                        <div class="command-title">{cmd.title.clone()}</div>
                                                        {cmd.subtitle.clone().map(|s| view! {
                                                            <div class="command-subtitle">{s}</div>
                                                        })}
                                                    </div>
                                                    {is_selected.then(|| view! {
                                                        <span class="command-enter-hint">"↵"</span>
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
                
                <div class="command-footer">
                    <span class="footer-hint">
                        <kbd>"↑"</kbd><kbd>"↓"</kbd>" navigate"
                    </span>
                    <span class="footer-hint">
                        <kbd>"↵"</kbd>" select"
                    </span>
                    <span class="footer-hint">
                        <kbd>"esc"</kbd>" close"
                    </span>
                </div>
            </div>
        </Show>
    }
}

