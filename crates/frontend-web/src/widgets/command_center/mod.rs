//! The "Neural" Command Palette (Cmd+K)

use leptos::*;

#[component]
pub fn CommandCenter() -> impl IntoView {
    let (is_open, set_open) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    
    // Global Keyboard Listener (Cmd+K)
    window_event_listener(ev::keydown, move |ev| {
        if (ev.meta_key() || ev.ctrl_key()) && ev.key().to_lowercase() == "k" {
            ev.prevent_default();
            set_open.update(|v| *v = !*v);
        }
        if ev.key() == "Escape" {
            set_open.set(false);
        }
    });

    view! {
        <Show when=move || is_open.get()>
            // 1. The Blur Backdrop
            <div 
                class="fixed inset-0 z-[100] bg-void/60 backdrop-blur-md flex pt-[20vh] justify-center animate-fade-in"
                on:click=move |_| set_open.set(false)
            >
                // 2. The Glass Monolith
                <div 
                    class="w-full max-w-2xl h-fit glass-morphism rounded-2xl border border-white/10 shadow-[0_0_50px_rgba(0,0,0,0.5)] overflow-hidden animate-scale-in"
                    on:click=move |e| e.stop_propagation()
                >
                    // Input
                    <div class="flex items-center px-4 py-4 border-b border-white/5">
                        <i class="fa-solid fa-search text-zinc-500 mr-4 text-xl"></i>
                        <input 
                            type="text" 
                            placeholder="Type a command or search..." 
                            autofocus=true
                            class="flex-1 bg-transparent border-none outline-none text-xl text-white placeholder-zinc-600"
                            prop:value=search_query
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                        <kbd class="px-2 py-1 rounded border border-white/10 bg-white/5 text-[10px] font-bold text-zinc-500">"ESC"</kbd>
                    </div>

                    // Smart Actions
                    <div class="p-2 max-h-[50vh] overflow-y-auto">
                        <CommandGroup title="Suggested">
                            <CommandItem icon="fa-bolt" label="Create New Deal" shortcut="N D" />
                            <CommandItem icon="fa-users" label="View All Contacts" shortcut="G C" />
                        </CommandGroup>
                        
                        <CommandGroup title="Navigation">
                            <CommandItem icon="fa-chart-line" label="Go to Dashboard" shortcut="G D" />
                            <CommandItem icon="fa-building" label="Go to Properties" shortcut="G P" />
                            <CommandItem icon="fa-inbox" label="Go to Inbox" shortcut="G I" />
                        </CommandGroup>
                        
                        <CommandGroup title="Actions">
                            <CommandItem icon="fa-user-plus" label="Add Contact" shortcut="N C" />
                            <CommandItem icon="fa-building-circle-plus" label="Add Property" shortcut="N P" />
                        </CommandGroup>
                    </div>
                    
                    // Footer
                    <div class="px-4 py-3 border-t border-white/5 flex items-center justify-between text-[10px] text-zinc-500">
                        <span>"Neural Core Online"</span>
                        <div class="flex items-center gap-2">
                            <span>"↑↓ Navigate"</span>
                            <span>"↵ Select"</span>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn CommandGroup(
    title: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="mb-2">
            <div class="px-3 py-2 text-[10px] uppercase tracking-widest text-zinc-600 font-bold">{title}</div>
            {children()}
        </div>
    }
}

#[component]
fn CommandItem(
    icon: &'static str,
    label: &'static str,
    shortcut: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-white/5 cursor-pointer transition-colors group">
            <i class=format!("fa-solid {} text-zinc-500 group-hover:text-violet-400 transition-colors w-5", icon) />
            <span class="flex-1 text-sm text-zinc-300 group-hover:text-white transition-colors">{label}</span>
            <kbd class="px-1.5 py-0.5 rounded border border-white/5 bg-white/5 text-[9px] font-mono text-zinc-500">{shortcut}</kbd>
        </div>
    }
}
