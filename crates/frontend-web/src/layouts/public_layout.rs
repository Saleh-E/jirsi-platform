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
    
    view! {
        <div class="public-layout">
            // Dynamic CSS Variables
            {move || branding.get().flatten().map(|b| {
                let css = format!(
                    ":root {{ --primary-color: {}; --secondary-color: {}; }}",
                    b.primary_color, b.secondary_color
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
.public-layout { min-height: 100vh; display: flex; flex-direction: column; background: var(--bg-primary, #0f0f14); color: var(--text-primary, #f0f0f5); }
.public-header { background: var(--bg-secondary, #1a1a24); border-bottom: 1px solid var(--border-color, #2a2a3a); padding: 1rem 0; position: sticky; top: 0; z-index: 100; }
.public-header__container { max-width: 1280px; margin: 0 auto; padding: 0 1.5rem; display: flex; align-items: center; justify-content: space-between; }
.public-header__logo { display: flex; align-items: center; text-decoration: none; color: inherit; }
.public-header__logo-img { max-height: 40px; width: auto; }
.public-header__logo-text { font-size: 1.5rem; font-weight: 700; color: var(--primary-color, #7c3aed); }
.public-header__nav { display: flex; align-items: center; gap: 1.5rem; }
.public-header__link { color: var(--text-secondary, #a0a0b0); text-decoration: none; font-weight: 500; transition: color 0.2s; }
.public-header__link:hover { color: var(--primary-color, #7c3aed); }
.public-header__btn { padding: 0.5rem 1.25rem; background: var(--primary-color, #7c3aed); color: white; border-radius: 8px; text-decoration: none; font-weight: 500; transition: opacity 0.2s; }
.public-header__btn:hover { opacity: 0.9; }
.public-main { flex: 1; max-width: 1280px; margin: 0 auto; padding: 2rem 1.5rem; width: 100%; }
.public-footer { background: var(--bg-secondary, #1a1a24); border-top: 1px solid var(--border-color, #2a2a3a); padding: 2rem 0; margin-top: auto; }
.public-footer__container { max-width: 1280px; margin: 0 auto; padding: 0 1.5rem; display: flex; flex-direction: column; gap: 1rem; align-items: center; text-align: center; }
.public-footer__info { display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center; }
.public-footer__name { font-weight: 600; color: var(--primary-color, #7c3aed); }
.public-footer__address, .public-footer__phone, .public-footer__email { color: var(--text-muted, #888); }
.public-footer__phone, .public-footer__email { text-decoration: none; }
.public-footer__phone:hover, .public-footer__email:hover { color: var(--primary-color, #7c3aed); }
.public-footer__copyright { color: var(--text-muted, #666); font-size: 0.875rem; }
        "#}
        </style>
    }
}
