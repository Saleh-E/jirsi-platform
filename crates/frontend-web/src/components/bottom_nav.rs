//! Mobile Bottom Navigation Component

use leptos::*;
use leptos_router::*;

/// Bottom navigation bar for mobile devices
#[component]
pub fn BottomNav() -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();
    
    // Clone navigate for each button
    let nav_home = navigate.clone();
    let nav_leads = navigate.clone();
    let nav_listings = navigate.clone();
    let nav_inbox = navigate.clone();
    let nav_more = navigate;
    
    // Check active route
    let is_active = move |path: &str| -> bool {
        location.pathname.get().starts_with(path)
    };
    
    view! {
        <nav class="bottom-nav">
            <button 
                class="nav-btn"
                class:active=move || is_active("/app/dashboard") || location.pathname.get() == "/app"
                on:click=move |_| nav_home("/app/dashboard", Default::default())
            >
                <span class="nav-icon">"🏠"</span>
                <span class="nav-label">"Home"</span>
            </button>
            
            <button 
                class="nav-btn"
                class:active=move || is_active("/app/crm")
                on:click=move |_| nav_leads("/app/crm/entity/contact", Default::default())
            >
                <span class="nav-icon">"👥"</span>
                <span class="nav-label">"Leads"</span>
            </button>
            
            <button 
                class="nav-btn"
                class:active=move || is_active("/app/realestate")
                on:click=move |_| nav_listings("/app/realestate/entity/property", Default::default())
            >
                <span class="nav-icon">"🏘️"</span>
                <span class="nav-label">"Listings"</span>
            </button>
            
            <button 
                class="nav-btn"
                class:active=move || is_active("/app/inbox")
                on:click=move |_| nav_inbox("/app/inbox", Default::default())
            >
                <span class="nav-icon">"📬"</span>
                <span class="nav-label">"Inbox"</span>
            </button>
            
            <button 
                class="nav-btn"
                class:active=move || is_active("/app/settings")
                on:click=move |_| nav_more("/app/settings", Default::default())
            >
                <span class="nav-icon">"☰"</span>
                <span class="nav-label">"More"</span>
            </button>
        </nav>
    }
}
