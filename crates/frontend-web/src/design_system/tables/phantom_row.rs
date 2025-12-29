//! Phantom Row: Magical hover interactions for table rows.
//! Actions are invisible until mouse hovers OR element is focused (A11y).

use leptos::*;

#[component]
pub fn PhantomRow(
    #[prop(optional)]
    children: Option<Children>,
    #[prop(optional)]
    on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(default = false)]
    is_header: bool,
    
    // Structured Props (Optional)
    #[prop(optional, into)]
    primary_text: Option<String>,
    #[prop(optional, into)]
    secondary_text: Option<String>,
    #[prop(optional, into)]
    status: Option<String>,
    #[prop(optional, into)]
    status_color: Option<String>,
    #[prop(optional, into)]
    meta_text: Option<String>,
) -> impl IntoView {
    let (is_hovered, set_hovered) = create_signal(false);

    // Added focus-within for accessibility
    let base_class = "group relative flex items-center px-4 py-3 border-b border-white/5 transition-all duration-300 ease-spring focus-within:bg-white/5 focus-within:pl-6";
    let hover_class = "hover:bg-white/5 hover:pl-6 cursor-pointer";
    
    // Determine content: Use children if provided, otherwise use structured props
    let content = if let Some(c) = children {
        c().into_view()
    } else {
         view! {
            // Structured Content Fallback
            <div class="flex-1">
                <div class="font-medium text-white">{primary_text.clone().unwrap_or_default()}</div>
                {secondary_text.clone().map(|t| view! { <div class="text-xs text-zinc-500">{t}</div> })}
            </div>
            
            // Status Badge
            {status.clone().map(|s| {
                let color = status_color.clone().unwrap_or("zinc".to_string());
                let bg = format!("bg-{}-500/10", color);
                let text = format!("text-{}-400", color);
                let border = format!("border-{}-500/20", color);
                view! {
                    <div class=format!("px-2 py-0.5 rounded text-[10px] uppercase font-bold border {} {} {}", border, bg, text)>
                        {s}
                    </div>
                }
            })}
            
            // Meta Value
            {meta_text.clone().map(|m| view! {
                <div class="text-right font-mono text-zinc-400">{m}</div>
            })}
        }.into_view()
    };
    
    view! {
        <div 
            class=format!("{} {}", base_class, if !is_header { hover_class } else { "" })
            on:mouseenter=move |_| set_hovered.set(true)
            on:mouseleave=move |_| set_hovered.set(false)
            on:click=move |ev| {
                if let Some(cb) = on_click {
                    cb.call(ev);
                }
            }
            tabindex="0"
        >
            // The "Phantom" Glow Indicator
            <div 
                class="absolute left-0 top-0 bottom-0 w-1 bg-violet-500 shadow-[0_0_15px_var(--accent-glow)] opacity-0 transition-opacity duration-300 group-focus-within:opacity-100"
                class:opacity-100=move || is_hovered.get() && !is_header
            />

            // Content Area
            <div class="flex-1 flex items-center gap-4 text-sm text-zinc-300 group-hover:text-white group-focus-within:text-white transition-colors">
                {content}
            </div>

            // Phantom Actions (Slide in from right)
            <div 
                class="flex items-center gap-2 opacity-0 blur-sm transform translate-x-4 transition-all duration-300 group-hover:opacity-100 group-hover:blur-0 group-hover:translate-x-0 group-focus-within:opacity-100 group-focus-within:blur-0 group-focus-within:translate-x-0"
            >
                <PhantomAction icon="fa-pen" label="Edit" intent="neutral" />
                <PhantomAction icon="fa-trash" label="Delete" intent="danger" />
            </div>
        </div>
    }
}

#[component]
fn PhantomAction(
    icon: &'static str, 
    label: &'static str,
    #[prop(default = "neutral")]
    intent: &'static str,
) -> impl IntoView {
    let color_class = if intent == "danger" {
        "hover:bg-red-500/20 hover:text-red-400"
    } else {
        "hover:bg-white/10 hover:text-white"
    };
    
    view! {
        <button 
            class=format!("w-7 h-7 rounded-lg flex items-center justify-center text-zinc-500 transition-all duration-200 {}", color_class)
            title=label
        >
            <i class=format!("fa-solid {} text-xs", icon) />
        </button>
    }
}
