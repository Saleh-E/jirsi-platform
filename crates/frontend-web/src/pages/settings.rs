//! Settings Page - Application settings management

use leptos::*;
use wasm_bindgen::JsCast;

/// Settings page with tabbed interface
#[component]
pub fn SettingsPage() -> impl IntoView {
    // Active tab state
    let (active_tab, set_active_tab) = create_signal("general".to_string());
    
    // Check admin role (from localStorage)
    let is_admin = move || {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("user_role").ok())
            .flatten()
            .map(|r| r == "admin")
            .unwrap_or(true) // Default to true for demo
    };

    view! {
        <div class="settings-page">
            <div class="settings-header">
                <h1 class="settings-title">"Settings"</h1>
                <p class="settings-subtitle">"Manage your application settings"</p>
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
                {move || is_admin().then(|| view! {
                    <button
                        class=move || format!("settings-tab {}", if active_tab.get() == "users" { "active" } else { "" })
                        on:click=move |_| set_active_tab.set("users".to_string())
                    >
                        <span class="tab-icon">"👥"</span>
                        "Users"
                    </button>
                })}
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
    }
}

/// General settings tab
#[component]
fn GeneralSettings() -> impl IntoView {
    let (tenant_name, set_tenant_name) = create_signal("Demo Real Estate".to_string());
    let (timezone, set_timezone) = create_signal("UTC".to_string());
    let (currency, set_currency) = create_signal("USD".to_string());
    let (saving, set_saving) = create_signal(false);
    let (saved, set_saved) = create_signal(false);

    let on_save = move |_| {
        set_saving.set(true);
        set_saved.set(false);
        
        set_timeout(
            move || {
                // Save to localStorage
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item("tenant_name", &tenant_name.get());
                    let _ = storage.set_item("timezone", &timezone.get());
                    let _ = storage.set_item("currency", &currency.get());
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
            <p class="section-description">"Configure your organization details"</p>

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
                        let style = root.unchecked_ref::<web_sys::HtmlElement>().style();
                        let _ = style.set_property("--primary", &primary_color.get());
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

/// Users settings tab (Admin only)
#[component]
fn UsersSettings() -> impl IntoView {
    view! {
        <div class="settings-section">
            <h2 class="section-title">"User Management"</h2>
            <p class="section-description">"Manage team members and permissions"</p>

            <div class="users-placeholder">
                <div class="placeholder-icon">"👥"</div>
                <h3>"User Management Coming Soon"</h3>
                <p>"This feature is currently under development. You'll soon be able to:"</p>
                <ul class="placeholder-features">
                    <li>"Invite new team members"</li>
                    <li>"Assign roles and permissions"</li>
                    <li>"Manage user access"</li>
                    <li>"View activity logs"</li>
                </ul>
            </div>
        </div>
    }
}
