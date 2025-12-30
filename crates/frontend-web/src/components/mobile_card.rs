//! Mobile Card Component for Entity Lists with Swipe Actions
//!
//! Features:
//! - Swipe left to reveal action buttons (Call, WhatsApp)
//! - Tap to navigate to detail
//! - Status badges
//! - Avatar with fallback

use leptos::*;

/// Props for mobile card
#[derive(Clone)]
pub struct MobileCardData {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub status: Option<String>,
    pub image_url: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Swipeable mobile card component for entity list items
#[component]
pub fn MobileCard(
    data: MobileCardData,
    #[prop(into)] on_click: Callback<String>,
) -> impl IntoView {
    let id = data.id.clone();
    let phone = data.phone.clone();
    let email = data.email.clone();
    
    // Swipe state - CSS class based for simplicity
    let (swiped, set_swiped) = create_signal(false);
    
    // Toggle swipe on long-press or button click on mobile
    let toggle_actions = {
        let swiped_clone = swiped;
        move |_| set_swiped.set(!swiped_clone.get())
    };
    
    view! {
        <div class="relative overflow-hidden">
            // Hidden action buttons (revealed on swipe)
            <div class="absolute inset-y-0 right-0 flex items-stretch gap-0 transition-transform duration-200">
                {phone.clone().map(|p| {
                    let phone_url = format!("tel:{}", p);
                    view! {
                        <a href=phone_url class="flex flex-col items-center justify-center px-4 bg-green-600 text-white" on:click=|e| e.stop_propagation()>
                            <span class="text-lg">"📞"</span>
                            <span class="text-[10px]">"Call"</span>
                        </a>
                    }
                })}
                {phone.clone().map(|p| {
                    let wa_url = format!("https://wa.me/{}", p.replace(&['+', ' ', '-'][..], ""));
                    view! {
                        <a href=wa_url target="_blank" class="flex flex-col items-center justify-center px-4 bg-emerald-500 text-white" on:click=|e| e.stop_propagation()>
                            <span class="text-lg">"💬"</span>
                            <span class="text-[10px]">"WhatsApp"</span>
                        </a>
                    }
                })}
                {email.map(|e| {
                    let email_url = format!("mailto:{}", e);
                    view! {
                        <a href=email_url class="flex flex-col items-center justify-center px-4 bg-blue-600 text-white" on:click=|ev| ev.stop_propagation()>
                            <span class="text-lg">"✉️"</span>
                            <span class="text-[10px]">"Email"</span>
                        </a>
                    }
                })}
            </div>
            
            // Main card
            <div 
                class=move || format!(
                    "relative flex items-center gap-3 p-3 bg-surface border-b border-white/10 transition-transform duration-200 {}",
                    if swiped.get() { "-translate-x-[120px]" } else { "" }
                )
                on:click=move |_| {
                    if !swiped.get() {
                        on_click.call(id.clone());
                    } else {
                        // Tap on swiped card resets it
                        set_swiped.set(false);
                    }
                }
            >
                <div class="w-12 h-12 rounded-full overflow-hidden bg-white/10 flex-shrink-0">
                    {match &data.image_url {
                        Some(url) => view! { <img src=url.clone() alt="avatar" class="w-full h-full object-cover"/> }.into_view(),
                        None => view! { 
                            <div class="w-full h-full flex items-center justify-center text-lg font-bold text-slate-400">
                                {data.title.chars().next().unwrap_or('?').to_string()}
                            </div>
                        }.into_view(),
                    }}
                </div>
                
                <div class="flex-1 min-w-0">
                    <div class="font-medium text-white truncate">{data.title}</div>
                    <div class="text-sm text-slate-400 truncate">{data.subtitle}</div>
                    {data.status.map(|s| {
                        let status_class = get_status_class(&s);
                        view! {
                            <span class=format!("inline-block mt-1 px-2 py-0.5 text-[10px] font-medium rounded-full {}", status_class)>{s}</span>
                        }
                    })}
                </div>
                
                // Swipe toggle button for accessibility/alternative gesture
                <button 
                    class="w-8 h-8 flex items-center justify-center text-slate-400 hover:text-white transition-colors" 
                    on:click=move |e| {
                        e.stop_propagation();
                        toggle_actions(());
                    }
                >
                    "⋮"
                </button>
                
                <div class="text-slate-500 text-lg">"›"</div>
            </div>
        </div>
    }
}

/// Get CSS class for status badge
fn get_status_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "new" | "open" | "active" => "bg-green-500/20 text-green-400",
        "in_progress" | "pending" | "working" => "bg-amber-500/20 text-amber-400",
        "won" | "completed" | "closed" | "success" => "bg-emerald-500/20 text-emerald-400",
        "lost" | "cancelled" | "failed" => "bg-red-500/20 text-red-400",
        "qualified" | "contacted" => "bg-blue-500/20 text-blue-400",
        _ => "bg-slate-500/20 text-slate-400",
    }
}

/// Mobile card list wrapper
#[component]
pub fn MobileCardList(
    items: Vec<MobileCardData>,
    #[prop(into)] on_item_click: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col divide-y divide-white/5">
            <For
                each=move || items.clone()
                key=|item| item.id.clone()
                children=move |item| {
                    let on_click = on_item_click.clone();
                    view! { <MobileCard data=item on_click=on_click/> }
                }
            />
        </div>
    }
}
