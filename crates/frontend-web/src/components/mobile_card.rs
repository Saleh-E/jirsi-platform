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
        <div class="mobile-card-container">
            // Hidden action buttons (revealed on swipe)
            <div class="mobile-card-actions">
                {phone.clone().map(|p| {
                    let phone_url = format!("tel:{}", p);
                    view! {
                        <a href=phone_url class="swipe-action call-action" on:click=|e| e.stop_propagation()>
                            <span class="action-icon">"📞"</span>
                            <span class="action-label">"Call"</span>
                        </a>
                    }
                })}
                {phone.clone().map(|p| {
                    let wa_url = format!("https://wa.me/{}", p.replace(&['+', ' ', '-'][..], ""));
                    view! {
                        <a href=wa_url target="_blank" class="swipe-action whatsapp-action" on:click=|e| e.stop_propagation()>
                            <span class="action-icon">"💬"</span>
                            <span class="action-label">"WhatsApp"</span>
                        </a>
                    }
                })}
                {email.map(|e| {
                    let email_url = format!("mailto:{}", e);
                    view! {
                        <a href=email_url class="swipe-action email-action" on:click=|ev| ev.stop_propagation()>
                            <span class="action-icon">"✉️"</span>
                            <span class="action-label">"Email"</span>
                        </a>
                    }
                })}
            </div>
            
            // Main card
            <div 
                class="mobile-card" 
                class:swiped=move || swiped.get()
                on:click=move |_| {
                    if !swiped.get() {
                        on_click.call(id.clone());
                    } else {
                        // Tap on swiped card resets it
                        set_swiped.set(false);
                    }
                }
            >
                <div class="card-avatar">
                    {match &data.image_url {
                        Some(url) => view! { <img src=url.clone() alt="avatar"/> }.into_view(),
                        None => view! { 
                            <div class="avatar-placeholder">
                                {data.title.chars().next().unwrap_or('?').to_string()}
                            </div>
                        }.into_view(),
                    }}
                </div>
                
                <div class="card-content">
                    <div class="card-title">{data.title}</div>
                    <div class="card-subtitle">{data.subtitle}</div>
                    {data.status.map(|s| {
                        let status_class = get_status_class(&s);
                        view! {
                            <span class=format!("card-status {}", status_class)>{s}</span>
                        }
                    })}
                </div>
                
                // Swipe toggle button for accessibility/alternative gesture
                <button 
                    class="card-more-btn" 
                    on:click=move |e| {
                        e.stop_propagation();
                        toggle_actions(());
                    }
                >
                    "⋮"
                </button>
                
                <div class="card-chevron">"›"</div>
            </div>
        </div>
    }
}

/// Get CSS class for status badge
fn get_status_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "new" | "open" | "active" => "status-new",
        "in_progress" | "pending" | "working" => "status-progress",
        "won" | "completed" | "closed" | "success" => "status-won",
        "lost" | "cancelled" | "failed" => "status-lost",
        "qualified" | "contacted" => "status-qualified",
        _ => "status-default",
    }
}

/// Mobile card list wrapper
#[component]
pub fn MobileCardList(
    items: Vec<MobileCardData>,
    #[prop(into)] on_item_click: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="mobile-card-list">
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
