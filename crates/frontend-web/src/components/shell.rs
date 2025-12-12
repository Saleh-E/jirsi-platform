//! Application Shell - sidebar, topbar, and User menu

use leptos::*;
use leptos_router::*;

/// Main application shell with sidebar and topbar
#[component]
pub fn Shell() -> impl IntoView {
    let (show_user_menu, set_show_user_menu) = create_signal(false);
    let navigate = use_navigate();
    
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
        }
        navigate("/login", Default::default());
    };

    view! {
        <div class="app-shell">
            <aside class="sidebar">
                <div class="sidebar-header">
                    <h1 class="logo">"SaaS Platform"</h1>
                </div>
                <nav class="sidebar-nav">
                    <a href="/" class="nav-item">"Dashboard"</a>
                    <div class="nav-section">
                        <span class="nav-section-title">"CRM"</span>
                        <a href="/app/crm/entity/contact" class="nav-item">"Contacts"</a>
                        <a href="/app/crm/entity/company" class="nav-item">"Companies"</a>
                        <a href="/app/crm/entity/deal" class="nav-item">"Deals"</a>
                        <a href="/app/crm/entity/task" class="nav-item">"Tasks"</a>
                    </div>
                    <div class="nav-section">
                        <span class="nav-section-title">"Real Estate"</span>
                        <a href="/app/realestate/entity/property" class="nav-item">"🏠 Properties"</a>
                        <a href="/app/realestate/entity/viewing" class="nav-item">"📅 Viewings"</a>
                        <a href="/app/realestate/entity/offer" class="nav-item">"📝 Offers"</a>
                    </div>
                </nav>
            </aside>
            <main class="main-content">
                <header class="topbar">
                    <div class="search-bar">
                        <input type="text" placeholder="Search..." class="search-input"/>
                    </div>
                    <div class="user-menu-container">
                        <button class="user-btn" on:click=move |_| set_show_user_menu.update(|v| *v = !*v)>
                            {user_email}
                            <span class="arrow">"▼"</span>
                        </button>
                        {move || show_user_menu.get().then(|| view! {
                            <div class="user-dropdown">
                                <a href="/profile" class="dropdown-item">"Profile"</a>
                                <a href="/settings" class="dropdown-item">"Settings"</a>
                                <hr/>
                                <button class="dropdown-item logout" on:click=on_logout.clone()>
                                    "Logout"
                                </button>
                            </div>
                        })}
                    </div>
                </header>
                <div class="content">
                    <Outlet/>
                </div>
            </main>
        </div>
    }
}
