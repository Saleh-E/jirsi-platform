//! Public Layout - Tenant-branded layout for public website
//!
//! A minimal layout without sidebar/auth for public-facing pages.
//! Displays tenant branding (logo, colors) fetched from API.

use leptos::*;
use leptos_router::Outlet;
use serde::{Deserialize, Serialize};
use crate::api::fetch_json;

/// Tenant branding information
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TenantBranding {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default = "default_primary_color")]
    pub primary_color: String,
    #[serde(default = "default_secondary_color")]
    pub secondary_color: String,
    #[serde(default)]
    pub listing_page_title: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

fn default_primary_color() -> String { "#7c3aed".to_string() }
fn default_secondary_color() -> String { "#6366f1".to_string() }

/// Public Layout Component
#[component]
pub fn PublicLayout() -> impl IntoView {
    // Fetch tenant branding
    let branding = create_resource(
        || (),
        |_| async move {
            let url = "http://localhost:3000/public/tenant?tenant_slug=demo";
            fetch_json::<TenantBranding>(url).await.ok()
        }
    );
    
    // Theme state - check localStorage on init
    let initial_light = {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.get_item("public-theme").ok().flatten() == Some("light".to_string())
            } else { false }
        } else { false }
    };
    let (is_light_mode, set_is_light_mode) = create_signal(initial_light);
    
    // Toggle theme handler
    let toggle_theme = move |_| {
        let new_value = !is_light_mode.get();
        set_is_light_mode.set(new_value);
        // Persist to localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("public-theme", if new_value { "light" } else { "dark" });
            }
        }
    };
    
    view! {
        <div class="public-layout" class:light-mode=is_light_mode>
            // 🎨 CHAMELEON ENGINE: Dynamic Brand CSS Variables
            // Injects tenant branding as CSS custom properties for instant theming
            {move || branding.get().flatten().map(|b| {
                // Generate full brand color palette from primary/secondary
                let primary = &b.primary_color;
                let secondary = &b.secondary_color;
                
                // Chameleon Engine CSS - transforms entire UI based on tenant branding
                let css = format!(
                    r#":root {{
    /* CHAMELEON ENGINE - Tenant Brand Colors */
    --color-brand-primary: {primary};
    --color-brand-secondary: {secondary};
    --color-brand-gradient: linear-gradient(135deg, {primary} 0%, {secondary} 100%);
    --color-brand-gradient-hover: linear-gradient(135deg, {secondary} 0%, {primary} 100%);
    
    /* Derived from brand (alpha variations) */
    --color-brand-primary-10: {primary}1a;
    --color-brand-primary-20: {primary}33;
    --color-brand-primary-50: {primary}80;
    --color-brand-secondary-10: {secondary}1a;
    --color-brand-secondary-20: {secondary}33;
    
    /* Brand shadows and glows */
    --shadow-brand: 0 4px 20px {primary}40;
    --shadow-brand-lg: 0 8px 32px {primary}50;
    --glow-brand: 0 0 20px {primary}60;
    
    /* Legacy compatibility */
    --primary-color: {primary};
    --secondary-color: {secondary};
}}"#,
                    primary = primary,
                    secondary = secondary,
                );
                view! { <style>{css}</style> }
            })}
            
            // Header
            <header class="public-header">
                <div class="public-header__container">
                    <a href="/" class="public-header__logo">
                        {move || branding.get().flatten().map(|b| {
                            if let Some(logo) = b.logo_url {
                                view! { <img src=logo alt=b.name.clone() class="public-header__logo-img" /> }.into_view()
                            } else {
                                view! { <span class="public-header__logo-text">{b.name}</span> }.into_view()
                            }
                        })}
                    </a>
                    
                    <nav class="public-header__nav">
                        <a href="/listings" class="public-header__link">"Properties"</a>
                        // Theme toggle button
                        <button 
                            class="theme-toggle"
                            id="theme-toggle-btn"
                            title="Toggle dark/light mode"
                            on:click=toggle_theme
                        >
                            <span class="theme-toggle__icon theme-toggle__sun">"☀️"</span>
                            <span class="theme-toggle__icon theme-toggle__moon">"🌙"</span>
                        </button>
                        <a href="#contact" class="public-header__btn">"Contact Us"</a>
                    </nav>
                </div>
            </header>
            
            // Main Content (Outlet for nested routes)
            <main class="public-main">
                <Outlet/>
            </main>
            
            // Footer
            <footer class="public-footer">
                <div class="public-footer__container">
                    {move || branding.get().flatten().map(|b| {
                        view! {
                            <div class="public-footer__info">
                                <span class="public-footer__name">{b.name.clone()}</span>
                                {b.address.map(|addr| view! {
                                    <span class="public-footer__address">{addr}</span>
                                })}
                                {b.phone.map(|phone| view! {
                                    <a href=format!("tel:{}", phone) class="public-footer__phone">{phone}</a>
                                })}
                                {b.email.map(|email| view! {
                                    <a href=format!("mailto:{}", email) class="public-footer__email">{email}</a>
                                })}
                            </div>
                        }
                    })}
                    <div class="public-footer__copyright">
                        {"© 2024 All Rights Reserved"}
                    </div>
                </div>
            </footer>
        </div>
        
        // CSS
        <style>
        {r#"
/* Theme Toggle Icon Animations - Keep only these since they need .light-mode context */
.theme-toggle__icon {
    font-size: 1.25rem;
    position: absolute;
    transition: all 0.3s ease;
}

.theme-toggle__sun {
    opacity: 0;
    transform: translateY(20px) rotate(180deg);
}

.theme-toggle__moon {
    opacity: 1;
    transform: translateY(0) rotate(0deg);
}

.light-mode .theme-toggle__sun {
    opacity: 1;
    transform: translateY(0) rotate(0deg);
}

.light-mode .theme-toggle__moon {
    opacity: 0;
    transform: translateY(-20px) rotate(-180deg);
}
        "#}
        </style>
        
        // Theme Toggle JavaScript
        <script>
        {r#"
document.addEventListener('DOMContentLoaded', function() {
    const layout = document.querySelector('.public-layout');
    const toggleBtn = document.getElementById('theme-toggle-btn');
    
    // Check saved theme
    const savedTheme = localStorage.getItem('public-theme');
    if (savedTheme === 'light') {
        layout.classList.add('light-mode');
    }
    
    // Toggle handler
    if (toggleBtn) {
        toggleBtn.addEventListener('click', function() {
            layout.classList.toggle('light-mode');
            const isLight = layout.classList.contains('light-mode');
            localStorage.setItem('public-theme', isLight ? 'light' : 'dark');
        });
    }
});
        "#}
        </script>
    }
}
