//! Mobile Bottom Navigation Component with More Menu Drawer

use leptos::*;
use leptos_router::*;

/// Bottom navigation bar for mobile devices
#[component]
pub fn BottomNav() -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();
    let (show_more_menu, set_show_more_menu) = create_signal(false);
    
    // Clone navigate for each button
    let nav_home = navigate.clone();
    let nav_leads = navigate.clone();
    let nav_listings = navigate.clone();
    let nav_inbox = navigate.clone();
    
    // Check active route
    let is_active = move |path: &str| -> bool {
        location.pathname.get().starts_with(path)
    };
    
    view! {
        // More Menu Drawer Overlay
        {move || {
            let nav = navigate.clone();
            show_more_menu.get().then(move || {
                let nav1 = nav.clone();
                let nav2 = nav.clone();
                let nav3 = nav.clone();
                let nav4 = nav.clone();
                let nav5 = nav.clone();
                let nav6 = nav.clone();
                let nav7 = nav.clone();
                view! {
                    <div class="more-menu-overlay" on:click=move |_| set_show_more_menu.set(false)>
                        <div class="more-menu-drawer" on:click=move |ev| ev.stop_propagation()>
                            <div class="drawer-header">
                                <h3>"More"</h3>
                                <button class="close-btn" on:click=move |_| set_show_more_menu.set(false)>"×"</button>
                            </div>
                            <div class="drawer-items">
                                <button class="drawer-item" on:click=move |_| { 
                                    nav1.clone()("/app/profile", Default::default());
                                    set_show_more_menu.set(false);
                                }>
                                    <span class="item-icon">"👤"</span>
                                    <span class="item-label">"Profile"</span>
                                    <span class="item-arrow">"›"</span>
                                </button>
                                <button class="drawer-item" on:click=move |_| {
                                    nav2.clone()("/app/crm/entity/deal", Default::default());
                                    set_show_more_menu.set(false);
                                }>
                                    <span class="item-icon">"💰"</span>
                                    <span class="item-label">"Deals"</span>
                                    <span class="item-arrow">"›"</span>
                                </button>
                                <button class="drawer-item" on:click=move |_| {
                                    nav3.clone()("/app/crm/entity/task", Default::default());
                                    set_show_more_menu.set(false);
                                }>
                                    <span class="item-icon">"✓"</span>
                                    <span class="item-label">"Tasks"</span>
                                    <span class="item-arrow">"›"</span>
                                </button>
                                <button class="drawer-item" on:click=move |_| {
                                    nav4.clone()("/app/calendar", Default::default());
                                    set_show_more_menu.set(false);
                                }>
                                    <span class="item-icon">"📅"</span>
                                    <span class="item-label">"Calendar"</span>
                                    <span class="item-arrow">"›"</span>
                                </button>
                                <button class="drawer-item" on:click=move |_| {
                                    nav5.clone()("/app/reports", Default::default());
                                    set_show_more_menu.set(false);
                                }>
                                    <span class="item-icon">"📊"</span>
                                    <span class="item-label">"Reports"</span>
                                    <span class="item-arrow">"›"</span>
                                </button>
                                <div class="drawer-divider"></div>
                                <button class="drawer-item" on:click=move |_| {
                                    nav6.clone()("/app/settings", Default::default());
                                    set_show_more_menu.set(false);
                                }>
                                    <span class="item-icon">"⚙️"</span>
                                    <span class="item-label">"Settings"</span>
                                    <span class="item-arrow">"›"</span>
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
        
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
                class:active=move || show_more_menu.get()
                on:click=move |_| set_show_more_menu.update(|v| *v = !*v)
            >
                <span class="nav-icon">"☰"</span>
                <span class="nav-label">"More"</span>
            </button>
        </nav>
    }
}
