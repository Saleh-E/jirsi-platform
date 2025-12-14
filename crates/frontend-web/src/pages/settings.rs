//! Settings Page - Application settings management with admin guard

use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};
use crate::api::{fetch_json, API_BASE, TENANT_ID};

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
                </div>

                // Tab Content
                <div class="settings-content">
                    {move || match active_tab.get().as_str() {
                        "general" => view! { <GeneralSettings/> }.into_view(),
                        "branding" => view! { <BrandingSettings/> }.into_view(),
                        "users" => view! { <UsersSettings/> }.into_view(),
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
    let (role, set_role) = create_signal("agent".to_string());
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
