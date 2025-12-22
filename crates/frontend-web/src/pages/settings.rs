//! Settings Page - Application settings management with admin guard

use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use crate::api::{fetch_json, patch_json, API_BASE, TENANT_ID};

/// Mock user data for the users table
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub last_active: Option<String>,
}

/// Settings page with tabbed interface and admin guard
#[component]
pub fn SettingsPage() -> impl IntoView {
    let navigate = use_navigate();
    
    // Active tab state
    let (active_tab, set_active_tab) = create_signal("general".to_string());
    
    // Check admin role (from localStorage)
    let (is_admin, set_is_admin) = create_signal(true); // Default true for demo
    let (checked_auth, set_checked_auth) = create_signal(false);
    
    // Check authorization on mount
    create_effect(move |_| {
        let admin = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("user_role").ok())
            .flatten()
            .map(|r| r == "admin" || r == "owner")
            .unwrap_or(true); // Default to true for demo
        
        set_is_admin.set(admin);
        set_checked_auth.set(true);
        
        // Redirect non-admins
        if !admin {
            navigate("/", Default::default());
        }
    });

    view! {
        <Show
            when=move || checked_auth.get() && is_admin.get()
            fallback=move || view! {
                <div class="settings-loading">
                    <div class="loading-spinner"></div>
                    <p>"Checking permissions..."</p>
                </div>
            }
        >
            <div class="settings-page">
                <div class="settings-header">
                    <h1 class="settings-title">"Settings"</h1>
                    <p class="settings-subtitle">"Manage your organization settings"</p>
                </div>

                // Tab Navigation
                <div class="settings-tabs">
                    <button
                        class=move || format!("settings-tab {}", if active_tab.get() == "general" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("general".to_string())
                    >
                        <span class="tab-icon">"⚙️"</span>
                        "General"
                    </button>
                    <button
                        class=move || format!("settings-tab {}", if active_tab.get() == "branding" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("branding".to_string())
                    >
                        <span class="tab-icon">"🎨"</span>
                        "Branding"
                    </button>
                    <button
                        class=move || format!("settings-tab {}", if active_tab.get() == "users" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("users".to_string())
                    >
                        <span class="tab-icon">"👥"</span>
                        "Team"
                    </button>
                    <button
                        class=move || format!("settings-tab {}", if active_tab.get() == "website" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("website".to_string())
                    >
                        <span class="tab-icon">"🌐"</span>
                        "Public Website"
                    </button>
                    <button
                        class=move || format!("settings-tab {}", if active_tab.get() == "integrations" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("integrations".to_string())
                    >
                        <span class="tab-icon">"🔗"</span>
                        "Integrations"
                    </button>
                </div>

                // Tab Content
                <div class="settings-content">
                    {move || match active_tab.get().as_str() {
                        "general" => view! { <GeneralSettings/> }.into_view(),
                        "branding" => view! { <BrandingSettings/> }.into_view(),
                        "users" => view! { <UsersSettings/> }.into_view(),
                        "website" => view! { <WebsiteSettings/> }.into_view(),
                        "integrations" => view! { <IntegrationsSettings/> }.into_view(),
                        _ => view! { <GeneralSettings/> }.into_view(),
                    }}
                </div>
            </div>
        </Show>
    }
}

/// General settings tab
#[component]
fn GeneralSettings() -> impl IntoView {
    let (tenant_name, set_tenant_name) = create_signal("Demo Real Estate".to_string());
    let (timezone, set_timezone) = create_signal("UTC".to_string());
    let (currency, set_currency) = create_signal("USD".to_string());
    let (locale, set_locale) = create_signal("en-US".to_string());
    let (saving, set_saving) = create_signal(false);
    let (saved, set_saved) = create_signal(false);

    // Load saved values
    create_effect(move |_| {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            if let Ok(Some(name)) = storage.get_item("tenant_name") {
                set_tenant_name.set(name);
            }
            if let Ok(Some(tz)) = storage.get_item("timezone") {
                set_timezone.set(tz);
            }
            if let Ok(Some(curr)) = storage.get_item("currency") {
                set_currency.set(curr);
            }
            if let Ok(Some(loc)) = storage.get_item("locale") {
                set_locale.set(loc);
            }
        }
    });

    let on_save = move |_| {
        set_saving.set(true);
        set_saved.set(false);
        
        set_timeout(
            move || {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item("tenant_name", &tenant_name.get());
                    let _ = storage.set_item("timezone", &timezone.get());
                    let _ = storage.set_item("currency", &currency.get());
                    let _ = storage.set_item("locale", &locale.get());
                }
                set_saving.set(false);
                set_saved.set(true);
            },
            std::time::Duration::from_millis(500),
        );
    };

    view! {
        <div class="settings-section">
            <h2 class="section-title">"General Settings"</h2>
            <p class="section-description">"Configure your organization details and preferences"</p>

            <div class="settings-form">
                <div class="form-field">
                    <label class="form-label">"Organization Name"</label>
                    <input
                        type="text"
                        class="form-input"
                        placeholder="Enter organization name"
                        prop:value=tenant_name
                        on:input=move |ev| set_tenant_name.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-row">
                    <div class="form-field">
                        <label class="form-label">"Timezone"</label>
                        <select
                            class="form-input form-select"
                            on:change=move |ev| set_timezone.set(event_target_value(&ev))
                        >
                            <option value="UTC" selected=move || timezone.get() == "UTC">"UTC"</option>
                            <option value="America/New_York" selected=move || timezone.get() == "America/New_York">"Eastern Time (US)"</option>
                            <option value="America/Los_Angeles" selected=move || timezone.get() == "America/Los_Angeles">"Pacific Time (US)"</option>
                            <option value="Europe/London" selected=move || timezone.get() == "Europe/London">"London"</option>
                            <option value="Europe/Paris" selected=move || timezone.get() == "Europe/Paris">"Paris"</option>
                            <option value="Asia/Dubai" selected=move || timezone.get() == "Asia/Dubai">"Dubai"</option>
                            <option value="Asia/Tokyo" selected=move || timezone.get() == "Asia/Tokyo">"Tokyo"</option>
                        </select>
                    </div>

                    <div class="form-field">
                        <label class="form-label">"Locale"</label>
                        <select
                            class="form-input form-select"
                            on:change=move |ev| set_locale.set(event_target_value(&ev))
                        >
                            <option value="en-US" selected=move || locale.get() == "en-US">"English (US)"</option>
                            <option value="en-GB" selected=move || locale.get() == "en-GB">"English (UK)"</option>
                            <option value="ar-AE" selected=move || locale.get() == "ar-AE">"Arabic (UAE)"</option>
                            <option value="fr-FR" selected=move || locale.get() == "fr-FR">"French"</option>
                            <option value="de-DE" selected=move || locale.get() == "de-DE">"German"</option>
                        </select>
                    </div>
                </div>

                <div class="form-field">
                    <label class="form-label">"Currency"</label>
                    <select
                        class="form-input form-select"
                        on:change=move |ev| set_currency.set(event_target_value(&ev))
                    >
                        <option value="USD" selected=move || currency.get() == "USD">"USD ($)"</option>
                        <option value="EUR" selected=move || currency.get() == "EUR">"EUR (€)"</option>
                        <option value="GBP" selected=move || currency.get() == "GBP">"GBP (£)"</option>
                        <option value="AED" selected=move || currency.get() == "AED">"AED (د.إ)"</option>
                        <option value="SAR" selected=move || currency.get() == "SAR">"SAR (ر.س)"</option>
                    </select>
                </div>

                {move || saved.get().then(|| view! {
                    <div class="profile-message success">"Settings saved successfully!"</div>
                })}

                <div class="settings-actions">
                    <button
                        class="btn btn-primary"
                        on:click=on_save
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Saving..." } else { "Save Settings" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Branding settings tab
#[component]
fn BrandingSettings() -> impl IntoView {
    let (logo_url, set_logo_url) = create_signal(String::new());
    let (primary_color, set_primary_color) = create_signal("#4f46e5".to_string());
    let (saving, set_saving) = create_signal(false);
    let (saved, set_saved) = create_signal(false);

    // Load saved values
    create_effect(move |_| {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            if let Ok(Some(logo)) = storage.get_item("brand_logo") {
                set_logo_url.set(logo);
            }
            if let Ok(Some(color)) = storage.get_item("brand_color") {
                set_primary_color.set(color);
            }
        }
    });

    let on_save = move |_| {
        set_saving.set(true);
        set_saved.set(false);
        
        set_timeout(
            move || {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item("brand_logo", &logo_url.get());
                    let _ = storage.set_item("brand_color", &primary_color.get());
                }
                
                // Apply color immediately
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(root) = document.document_element() {
                        let html: web_sys::HtmlElement = root.unchecked_into();
                        let _ = html.style().set_property("--primary", &primary_color.get());
                    }
                }
                
                set_saving.set(false);
                set_saved.set(true);
            },
            std::time::Duration::from_millis(500),
        );
    };

    view! {
        <div class="settings-section">
            <h2 class="section-title">"Branding"</h2>
            <p class="section-description">"Customize your brand appearance"</p>

            <div class="settings-form">
                <div class="form-field">
                    <label class="form-label">"Logo URL"</label>
                    <input
                        type="url"
                        class="form-input"
                        placeholder="https://example.com/logo.png"
                        prop:value=logo_url
                        on:input=move |ev| set_logo_url.set(event_target_value(&ev))
                    />
                    {move || (!logo_url.get().is_empty()).then(|| view! {
                        <div class="logo-preview">
                            <img src=logo_url.get() alt="Logo Preview" class="logo-preview-img"/>
                        </div>
                    })}
                </div>

                <div class="form-field">
                    <label class="form-label">"Primary Color"</label>
                    <div class="color-picker-row">
                        <input
                            type="color"
                            class="color-picker"
                            prop:value=primary_color
                            on:input=move |ev| set_primary_color.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            class="form-input color-input"
                            prop:value=primary_color
                            on:input=move |ev| set_primary_color.set(event_target_value(&ev))
                        />
                        <div
                            class="color-preview"
                            style=move || format!("background-color: {}", primary_color.get())
                        ></div>
                    </div>
                </div>

                // Color Presets
                <div class="form-field">
                    <label class="form-label">"Quick Presets"</label>
                    <div class="color-presets">
                        {["#4f46e5", "#0891b2", "#059669", "#d97706", "#dc2626", "#7c3aed", "#db2777", "#0d9488"]
                            .into_iter()
                            .map(|color| {
                                let c = color.to_string();
                                view! {
                                    <button
                                        class="color-preset"
                                        style=format!("background-color: {}", color)
                                        on:click=move |_| set_primary_color.set(c.clone())
                                    />
                                }
                            })
                            .collect_view()
                        }
                    </div>
                </div>

                {move || saved.get().then(|| view! {
                    <div class="profile-message success">"Branding saved successfully!"</div>
                })}

                <div class="settings-actions">
                    <button
                        class="btn btn-primary"
                        on:click=on_save
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Saving..." } else { "Save Branding" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Users/Team settings tab with member table
#[component]
fn UsersSettings() -> impl IntoView {
    let (show_invite_modal, set_show_invite_modal) = create_signal(false);
    
    // Mock team members data
    let team_members = vec![
        TeamMember {
            id: "1".to_string(),
            name: "John Admin".to_string(),
            email: "admin@demo.com".to_string(),
            role: "Owner".to_string(),
            status: "Active".to_string(),
            last_active: Some("Just now".to_string()),
        },
        TeamMember {
            id: "2".to_string(),
            name: "Sarah Manager".to_string(),
            email: "sarah@demo.com".to_string(),
            role: "Admin".to_string(),
            status: "Active".to_string(),
            last_active: Some("2 hours ago".to_string()),
        },
        TeamMember {
            id: "3".to_string(),
            name: "Mike Agent".to_string(),
            email: "mike@demo.com".to_string(),
            role: "Agent".to_string(),
            status: "Active".to_string(),
            last_active: Some("1 day ago".to_string()),
        },
        TeamMember {
            id: "4".to_string(),
            name: "Lisa Support".to_string(),
            email: "lisa@demo.com".to_string(),
            role: "Viewer".to_string(),
            status: "Invited".to_string(),
            last_active: None,
        },
    ];

    view! {
        <div class="settings-section">
            <div class="section-header-row">
                <div>
                    <h2 class="section-title">"Team Members"</h2>
                    <p class="section-description">"Manage your team and permissions"</p>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_invite_modal.set(true)
                >
                    "+ Invite User"
                </button>
            </div>

            <div class="users-table-container">
                <table class="users-table">
                    <thead>
                        <tr>
                            <th>"Member"</th>
                            <th>"Role"</th>
                            <th>"Status"</th>
                            <th>"Last Active"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {team_members.into_iter().map(|member| {
                            let role_class = match member.role.as_str() {
                                "Owner" => "role-badge role-badge--owner",
                                "Admin" => "role-badge role-badge--admin",
                                "Agent" => "role-badge role-badge--agent",
                                _ => "role-badge role-badge--viewer",
                            };
                            let status_class = if member.status == "Active" { "status-active" } else { "status-invited" };
                            
                            view! {
                                <tr>
                                    <td>
                                        <div class="member-info">
                                            <div class="member-avatar">
                                                {member.name.chars().next().unwrap_or('U')}
                                            </div>
                                            <div class="member-details">
                                                <span class="member-name">{&member.name}</span>
                                                <span class="member-email">{&member.email}</span>
                                            </div>
                                        </div>
                                    </td>
                                    <td>
                                        <span class=role_class>{&member.role}</span>
                                    </td>
                                    <td>
                                        <span class=status_class>{&member.status}</span>
                                    </td>
                                    <td class="text-muted">
                                        {member.last_active.clone().unwrap_or_else(|| "—".to_string())}
                                    </td>
                                    <td>
                                        <button class="btn-icon" title="Edit">
                                            "✏️"
                                        </button>
                                        <button class="btn-icon btn-icon--danger" title="Remove">
                                            "🗑️"
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            // Invite Modal
            {move || show_invite_modal.get().then(|| view! {
                <InviteUserModal on_close=move || set_show_invite_modal.set(false)/>
            })}
        </div>
    }
}

/// Invite user modal
#[component]
fn InviteUserModal(on_close: impl Fn() + 'static + Clone) -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (_role, set_role) = create_signal("agent".to_string());
    let (sending, set_sending) = create_signal(false);
    let (sent, set_sent) = create_signal(false);

    let on_close_clone1 = on_close.clone();
    let on_close_clone2 = on_close.clone();

    let on_submit = move |_| {
        if email.get().is_empty() {
            return;
        }
        
        set_sending.set(true);
        
        set_timeout(
            move || {
                set_sending.set(false);
                set_sent.set(true);
            },
            std::time::Duration::from_millis(500),
        );
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close()>
            <div class="modal-content" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Invite Team Member"</h2>
                    <button class="modal-close" on:click=move |_| on_close_clone1()>"×"</button>
                </div>
                <div class="modal-body">
                    {move || if sent.get() {
                        view! {
                            <div class="invite-success">
                                <span class="invite-success-icon">"✉️"</span>
                                <h3>"Invitation Sent!"</h3>
                                <p>"We've sent an invitation to "{email.get()}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <>
                                <div class="form-field">
                                    <label class="form-label">"Email Address"</label>
                                    <input
                                        type="email"
                                        class="form-input"
                                        placeholder="colleague@company.com"
                                        prop:value=email
                                        on:input=move |ev| set_email.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Role"</label>
                                    <select
                                        class="form-input form-select"
                                        on:change=move |ev| set_role.set(event_target_value(&ev))
                                    >
                                        <option value="admin">"Admin - Full access"</option>
                                        <option value="agent" selected=true>"Agent - Manage listings"</option>
                                        <option value="viewer">"Viewer - Read only"</option>
                                    </select>
                                </div>
                            </>
                        }.into_view()
                    }}
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" on:click=move |_| on_close_clone2()>
                        {move || if sent.get() { "Close" } else { "Cancel" }}
                    </button>
                    <Show when=move || !sent.get()>
                        <button
                            class="btn btn-primary"
                            on:click=on_submit
                            disabled=move || sending.get() || email.get().is_empty()
                        >
                            {move || if sending.get() { "Sending..." } else { "Send Invitation" }}
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// WEBSITE SETTINGS
// ============================================================================

/// Settings data structures matching backend
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TenantBranding {
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub favicon_url: Option<String>,
    #[serde(default = "default_primary_color")]
    pub primary_color: String,
    #[serde(default = "default_secondary_color")]
    pub secondary_color: String,
}

fn default_primary_color() -> String { "#7c3aed".to_string() }
fn default_secondary_color() -> String { "#6366f1".to_string() }

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TenantHero {
    #[serde(default)]
    pub headline: Option<String>,
    #[serde(default)]
    pub subtext: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TenantContact {
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TenantSettings {
    #[serde(default)]
    pub branding: TenantBranding,
    #[serde(default)]
    pub hero: TenantHero,
    #[serde(default)]
    pub contact: TenantContact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub tenant_id: String,
    pub tenant_name: String,
    pub subdomain: String,
    pub settings: TenantSettings,
}

/// Public Website settings tab
#[component]
fn WebsiteSettings() -> impl IntoView {
    // State for each field
    let (logo_url, set_logo_url) = create_signal(String::new());
    let (primary_color, set_primary_color) = create_signal("#7c3aed".to_string());
    let (secondary_color, set_secondary_color) = create_signal("#6366f1".to_string());
    let (hero_headline, set_hero_headline) = create_signal(String::new());
    let (hero_subtext, set_hero_subtext) = create_signal(String::new());
    let (hero_image, set_hero_image) = create_signal(String::new());
    let (contact_phone, set_contact_phone) = create_signal(String::new());
    let (contact_email, set_contact_email) = create_signal(String::new());
    let (contact_address, set_contact_address) = create_signal(String::new());
    
    // UI state
    let (loading, set_loading) = create_signal(true);
    let (saving, set_saving) = create_signal(false);
    let (saved, set_saved) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    
    // Load current settings on mount
    create_effect(move |_| {
        spawn_local(async move {
            let url = format!("{}/tenant/settings", API_BASE);
            match fetch_json::<SettingsResponse>(&url).await {
                Ok(res) => {
                    // Populate form fields
                    set_logo_url.set(res.settings.branding.logo_url.unwrap_or_default());
                    set_primary_color.set(res.settings.branding.primary_color);
                    set_secondary_color.set(res.settings.branding.secondary_color);
                    set_hero_headline.set(res.settings.hero.headline.unwrap_or_default());
                    set_hero_subtext.set(res.settings.hero.subtext.unwrap_or_default());
                    set_hero_image.set(res.settings.hero.image_url.unwrap_or_default());
                    set_contact_phone.set(res.settings.contact.phone.unwrap_or_default());
                    set_contact_email.set(res.settings.contact.email.unwrap_or_default());
                    set_contact_address.set(res.settings.contact.address.unwrap_or_default());
                }
                Err(_e) => {
                    // Use defaults if not authenticated or error
                }
            }
            set_loading.set(false);
        });
    });
    
    // Save settings
    let on_save = move |_| {
        set_saving.set(true);
        set_saved.set(false);
        set_error.set(None);
        
        let settings = TenantSettings {
            branding: TenantBranding {
                logo_url: if logo_url.get().is_empty() { None } else { Some(logo_url.get()) },
                favicon_url: None,
                primary_color: primary_color.get(),
                secondary_color: secondary_color.get(),
            },
            hero: TenantHero {
                headline: if hero_headline.get().is_empty() { None } else { Some(hero_headline.get()) },
                subtext: if hero_subtext.get().is_empty() { None } else { Some(hero_subtext.get()) },
                image_url: if hero_image.get().is_empty() { None } else { Some(hero_image.get()) },
            },
            contact: TenantContact {
                phone: if contact_phone.get().is_empty() { None } else { Some(contact_phone.get()) },
                email: if contact_email.get().is_empty() { None } else { Some(contact_email.get()) },
                address: if contact_address.get().is_empty() { None } else { Some(contact_address.get()) },
            },
        };
        
        spawn_local(async move {
            let url = format!("{}/tenant/settings", API_BASE);
            match patch_json::<_, SettingsResponse>(&url, &settings).await {
                Ok(_) => {
                    set_saved.set(true);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to save: {}", e)));
                }
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="section-title">"Public Website Settings"</h2>
            <p class="section-description">"Customize how your public website appears to visitors"</p>

            <Show when=move || loading.get()>
                <div class="settings-loading">
                    <div class="loading-spinner"></div>
                    <p>"Loading settings..."</p>
                </div>
            </Show>

            <Show when=move || !loading.get()>
                <div class="settings-form">
                    // BRANDING SECTION
                    <div class="form-section">
                        <h3 class="form-section-title">"🎨 Brand Colors"</h3>
                        
                        <div class="form-row">
                            <div class="form-field">
                                <label class="form-label">"Primary Color"</label>
                                <div class="color-picker-row">
                                    <input
                                        type="color"
                                        class="color-picker"
                                        prop:value=primary_color
                                        on:input=move |ev| set_primary_color.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="text"
                                        class="form-input color-input"
                                        prop:value=primary_color
                                        on:input=move |ev| set_primary_color.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Secondary Color"</label>
                                <div class="color-picker-row">
                                    <input
                                        type="color"
                                        class="color-picker"
                                        prop:value=secondary_color
                                        on:input=move |ev| set_secondary_color.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="text"
                                        class="form-input color-input"
                                        prop:value=secondary_color
                                        on:input=move |ev| set_secondary_color.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                        </div>

                        <div class="form-field">
                            <label class="form-label">"Logo URL"</label>
                            <input
                                type="url"
                                class="form-input"
                                placeholder="https://example.com/logo.png"
                                prop:value=logo_url
                                on:input=move |ev| set_logo_url.set(event_target_value(&ev))
                            />
                            {move || (!logo_url.get().is_empty()).then(|| view! {
                                <div class="logo-preview">
                                    <img src=logo_url.get() alt="Logo Preview" class="logo-preview-img"/>
                                </div>
                            })}
                        </div>
                    </div>

                    // HERO SECTION
                    <div class="form-section">
                        <h3 class="form-section-title">"🏠 Hero Section"</h3>
                        
                        <div class="form-field">
                            <label class="form-label">"Headline"</label>
                            <input
                                type="text"
                                class="form-input"
                                placeholder="Find Your Dream Home"
                                prop:value=hero_headline
                                on:input=move |ev| set_hero_headline.set(event_target_value(&ev))
                            />
                        </div>
                        
                        <div class="form-field">
                            <label class="form-label">"Subtext"</label>
                            <textarea
                                class="form-input form-textarea"
                                placeholder="Discover premium properties with our expert team..."
                                prop:value=hero_subtext
                                on:input=move |ev| set_hero_subtext.set(event_target_value(&ev))
                            />
                        </div>
                        
                        <div class="form-field">
                            <label class="form-label">"Hero Image URL"</label>
                            <input
                                type="url"
                                class="form-input"
                                placeholder="https://example.com/hero.jpg"
                                prop:value=hero_image
                                on:input=move |ev| set_hero_image.set(event_target_value(&ev))
                            />
                            {move || (!hero_image.get().is_empty()).then(|| view! {
                                <div class="hero-preview">
                                    <img src=hero_image.get() alt="Hero Preview" class="hero-preview-img"/>
                                </div>
                            })}
                        </div>
                    </div>

                    // CONTACT SECTION
                    <div class="form-section">
                        <h3 class="form-section-title">"📞 Contact Information"</h3>
                        
                        <div class="form-row">
                            <div class="form-field">
                                <label class="form-label">"Phone"</label>
                                <input
                                    type="tel"
                                    class="form-input"
                                    placeholder="+1 (555) 000-0000"
                                    prop:value=contact_phone
                                    on:input=move |ev| set_contact_phone.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Email"</label>
                                <input
                                    type="email"
                                    class="form-input"
                                    placeholder="info@yourcompany.com"
                                    prop:value=contact_email
                                    on:input=move |ev| set_contact_email.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        
                        <div class="form-field">
                            <label class="form-label">"Address"</label>
                            <input
                                type="text"
                                class="form-input"
                                placeholder="123 Main Street, City, State 12345"
                                prop:value=contact_address
                                on:input=move |ev| set_contact_address.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    // Messages
                    {move || error.get().map(|e| view! {
                        <div class="profile-message error">{e}</div>
                    })}
                    {move || saved.get().then(|| view! {
                        <div class="profile-message success">"Website settings saved successfully!"</div>
                    })}

                    // Actions
                    <div class="settings-actions">
                        <button
                            class="btn btn-primary"
                            on:click=on_save
                            disabled=move || saving.get()
                        >
                            {move || if saving.get() { "Saving..." } else { "Save Website Settings" }}
                        </button>
                        <a
                            href="/listings"
                            target="_blank"
                            class="btn btn-secondary"
                        >
                            "Preview Public Site →"
                        </a>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Integrations settings - Provider configuration cards
#[component]
fn IntegrationsSettings() -> impl IntoView {
    // Provider configurations
    let providers = vec![
        ("twilio", "Twilio (SMS/Voice)", "📱", "Send/receive SMS and voice calls"),
        ("facebook", "Facebook Lead Ads", "📘", "Sync leads from Facebook ads"),
        ("whatsapp", "WhatsApp Business", "💬", "Two-way WhatsApp messaging"),
        ("email", "Email (SMTP)", "📧", "Send transactional emails"),
    ];

    // State for each provider
    let (selected_provider, set_selected_provider) = create_signal::<Option<String>>(None);
    let (saving, set_saving) = create_signal(false);
    let (saved_msg, set_saved_msg) = create_signal::<Option<String>>(None);

    // Form fields for Twilio
    let (twilio_sid, set_twilio_sid) = create_signal(String::new());
    let (twilio_token, set_twilio_token) = create_signal(String::new());
    let (twilio_phone, set_twilio_phone) = create_signal(String::new());

    // Form fields for Facebook
    let (fb_app_id, set_fb_app_id) = create_signal(String::new());
    let (fb_secret, set_fb_secret) = create_signal(String::new());
    let (fb_page_token, set_fb_page_token) = create_signal(String::new());

    // Save handler
    let on_save = move |_| {
        set_saving.set(true);
        set_saved_msg.set(None);
        
        let provider = selected_provider.get().unwrap_or_default();
        let _tenant_id = TENANT_ID.to_string();
        
        spawn_local(async move {
            // TODO: Call API to save integration
            // For now, just simulate
            gloo_timers::future::TimeoutFuture::new(500).await;
            set_saving.set(false);
            set_saved_msg.set(Some(format!("{} configured successfully!", provider)));
            set_selected_provider.set(None);
        });
    };

    view! {
        <div class="settings-section integrations">
            <h2 class="section-title">"Integrations"</h2>
            <p class="section-description">"Connect external services to your CRM"</p>

            // Success message
            {move || saved_msg.get().map(|msg| view! {
                <div class="profile-message success">{msg}</div>
            })}

            // Provider cards grid
            <div class="integrations-grid">
                {providers.into_iter().map(|(id, name, icon, desc)| {
                    let _id_clone = id.to_string();
                    let id_for_click = id.to_string();
                    
                    view! {
                        <div class="integration-card">
                            <div class="integration-header">
                                <span class="integration-icon">{icon}</span>
                                <div class="integration-info">
                                    <h3 class="integration-name">{name}</h3>
                                    <p class="integration-desc">{desc}</p>
                                </div>
                            </div>
                            <div class="integration-status">
                                <span class="status-dot inactive"></span>
                                "Not Connected"
                            </div>
                            <button
                                class="btn btn-secondary btn-sm"
                                on:click=move |_| set_selected_provider.set(Some(id_for_click.clone()))
                            >
                                "Configure"
                            </button>
                        </div>
                    }
                }).collect_view()}
            </div>

            // Configuration modal
            <Show when=move || selected_provider.get().is_some()>
                <div class="modal-overlay" on:click=move |_| set_selected_provider.set(None)>
                    <div class="modal-content integration-modal" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h3>"Configure Integration"</h3>
                            <button class="modal-close" on:click=move |_| set_selected_provider.set(None)>
                                "×"
                            </button>
                        </div>
                        
                        <div class="modal-body">
                            {move || match selected_provider.get().as_deref() {
                                Some("twilio") => view! {
                                    <div class="form-group">
                                        <label>"Account SID"</label>
                                        <input
                                            type="text"
                                            placeholder="ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                                            prop:value=twilio_sid
                                            on:input=move |e| set_twilio_sid.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label>"Auth Token"</label>
                                        <input
                                            type="password"
                                            placeholder="Your auth token"
                                            prop:value=twilio_token
                                            on:input=move |e| set_twilio_token.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label>"Phone Number"</label>
                                        <input
                                            type="text"
                                            placeholder="+1234567890"
                                            prop:value=twilio_phone
                                            on:input=move |e| set_twilio_phone.set(event_target_value(&e))
                                        />
                                    </div>
                                }.into_view(),
                                Some("facebook") => view! {
                                    <div class="form-group">
                                        <label>"App ID"</label>
                                        <input
                                            type="text"
                                            placeholder="Your Facebook App ID"
                                            prop:value=fb_app_id
                                            on:input=move |e| set_fb_app_id.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label>"App Secret"</label>
                                        <input
                                            type="password"
                                            placeholder="Your App Secret"
                                            prop:value=fb_secret
                                            on:input=move |e| set_fb_secret.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label>"Page Access Token"</label>
                                        <textarea
                                            placeholder="Your page access token"
                                            prop:value=fb_page_token
                                            on:input=move |e| set_fb_page_token.set(event_target_value(&e))
                                        ></textarea>
                                    </div>
                                }.into_view(),
                                _ => view! {
                                    <p class="text-muted">"Configuration coming soon..."</p>
                                }.into_view(),
                            }}
                        </div>

                        <div class="modal-footer">
                            <button
                                class="btn btn-secondary"
                                on:click=move |_| set_selected_provider.set(None)
                            >
                                "Cancel"
                            </button>
                            <button
                                class="btn btn-primary"
                                on:click=on_save
                                disabled=saving
                            >
                                {move || if saving.get() { "Saving..." } else { "Save & Connect" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
