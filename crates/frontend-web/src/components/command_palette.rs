//! Command Palette Component
//!
//! A Ctrl+K activated command palette for global search and navigation.
//! Features: fuzzy search, keyboard navigation, categories.

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
    
    // Store commands statically
    let commands: Vec<CommandItem> = vec![
        CommandItem {
            id: "nav-contacts".to_string(),
            title: "Go to Contacts".to_string(),
            subtitle: Some("View all contacts".to_string()),
            category: "Navigation".to_string(),
            icon: "👤".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/contact".to_string()),
        },
        CommandItem {
            id: "nav-companies".to_string(),
            title: "Go to Companies".to_string(),
            subtitle: Some("View all companies".to_string()),
            category: "Navigation".to_string(),
            icon: "🏢".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/company".to_string()),
        },
        CommandItem {
            id: "nav-deals".to_string(),
            title: "Go to Deals".to_string(),
            subtitle: Some("View all deals".to_string()),
            category: "Navigation".to_string(),
            icon: "💰".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/deal".to_string()),
        },
        CommandItem {
            id: "nav-properties".to_string(),
            title: "Go to Properties".to_string(),
            subtitle: Some("View all properties".to_string()),
            category: "Navigation".to_string(),
            icon: "🏠".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/property".to_string()),
        },
        CommandItem {
            id: "nav-tasks".to_string(),
            title: "Go to Tasks".to_string(),
            subtitle: Some("View all tasks".to_string()),
            category: "Navigation".to_string(),
            icon: "✓".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/task".to_string()),
        },
        CommandItem {
            id: "nav-reports".to_string(),
            title: "Go to Reports".to_string(),
            subtitle: Some("View analytics".to_string()),
            category: "Navigation".to_string(),
            icon: "📊".to_string(),
            action: CommandAction::Navigate("/app/reports".to_string()),
        },
        CommandItem {
            id: "nav-inbox".to_string(),
            title: "Go to Inbox".to_string(),
            subtitle: Some("View messages".to_string()),
            category: "Navigation".to_string(),
            icon: "📧".to_string(),
            action: CommandAction::Navigate("/app/inbox".to_string()),
        },
        CommandItem {
            id: "action-new-contact".to_string(),
            title: "Create New Contact".to_string(),
            subtitle: Some("Add a new contact".to_string()),
            category: "Actions".to_string(),
            icon: "+".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/contact?create=true".to_string()),
        },
        CommandItem {
            id: "action-new-deal".to_string(),
            title: "Create New Deal".to_string(),
            subtitle: Some("Start a new deal".to_string()),
            category: "Actions".to_string(),
            icon: "+".to_string(),
            action: CommandAction::Navigate("/app/crm/entity/deal?create=true".to_string()),
        },
    ];
    
    // Store commands for reactive access
    let commands_store = store_value(commands);
    
    // Filter commands based on search
    let filtered_commands = move || {
        let query = search_query.get().to_lowercase();
        let cmds = commands_store.get_value();
        if query.is_empty() {
            cmds
        } else {
            cmds.into_iter()
                .filter(|cmd| {
                    cmd.title.to_lowercase().contains(&query) ||
                    cmd.subtitle.as_ref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false)
                })
                .collect()
        }
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
                        placeholder="Type a command or search..."
                        autofocus
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
                                    "No results found"
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
                                                >
                                                    <span class="command-icon">{cmd.icon.clone()}</span>
                                                    <div class="command-text">
                                                        <div class="command-title">{cmd.title.clone()}</div>
                                                        {cmd.subtitle.clone().map(|s| view! {
                                                            <div class="command-subtitle">{s}</div>
                                                        })}
                                                    </div>
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
                        <kbd>"↑"</kbd><kbd>"↓"</kbd>" to navigate"
                    </span>
                    <span class="footer-hint">
                        <kbd>"↵"</kbd>" to select"
                    </span>
                    <span class="footer-hint">
                        <kbd>"esc"</kbd>" to close"
                    </span>
                </div>
            </div>
        </Show>
    }
}
