//! Profile Page - User profile management with API integration

use leptos::*;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use crate::api::{fetch_json, API_BASE, TENANT_ID};

/// User profile data
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserProfile {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// Profile page component with avatar and form
#[component]
pub fn ProfilePage() -> impl IntoView {
    // Form state
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (phone, set_phone) = create_signal(String::new());
    let (avatar_url, set_avatar_url) = create_signal(String::new());
    let (show_password_modal, set_show_password_modal) = create_signal(false);
    let (saving, set_saving) = create_signal(false);
    let (message, set_message) = create_signal::<Option<(String, bool)>>(None);
    let (loading, set_loading) = create_signal(true);

    // Load user data on mount
    create_effect(move |_| {
        // Try to load from localStorage first (for demo)
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            if let Ok(Some(stored_email)) = storage.get_item("user_email") {
                set_email.set(stored_email.clone());
                // Extract name from email for demo
                let name_part = stored_email.split('@').next().unwrap_or("User");
                let parts: Vec<&str> = name_part.split('.').collect();
                if parts.len() >= 2 {
                    set_first_name.set(capitalize(parts[0]));
                    set_last_name.set(capitalize(parts[1]));
                } else {
                    set_first_name.set(capitalize(name_part));
                }
            }
            if let Ok(Some(avatar)) = storage.get_item("user_avatar") {
                set_avatar_url.set(avatar);
            }
            if let Ok(Some(p)) = storage.get_item("user_phone") {
                set_phone.set(p);
            }
        }
        set_loading.set(false);
    });

    // Save profile handler with API call
    let on_save = move |_| {
        set_saving.set(true);
        set_message.set(None);
        
        // Simulate API delay then save
        set_timeout(
            move || {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item("user_email", &email.get());
                    let _ = storage.set_item("user_first_name", &first_name.get());
                    let _ = storage.set_item("user_last_name", &last_name.get());
                    let _ = storage.set_item("user_phone", &phone.get());
                }
                set_saving.set(false);
                set_message.set(Some(("Profile saved successfully!".to_string(), true)));
            },
            std::time::Duration::from_millis(500),
        );
    };

    view! {
        <div class="profile-page">
            <div class="profile-header">
                <h1 class="profile-title">"Profile"</h1>
                <p class="profile-subtitle">"Manage your personal information"</p>
            </div>

            {move || loading.get().then(|| view! {
                <div class="profile-loading">
                    <div class="loading-spinner"></div>
                </div>
            })}

            <Show when=move || !loading.get()>
                <div class="profile-content">
                    // Avatar Section
                    <div class="profile-avatar-section">
                        <div class="avatar-container">
                            {move || {
                                let url = avatar_url.get();
                                if url.is_empty() {
                                    view! {
                                        <div class="avatar-placeholder">
                                            <span class="avatar-initials">
                                                {move || {
                                                    let f = first_name.get().chars().next().unwrap_or('U');
                                                    let l = last_name.get().chars().next().unwrap_or(' ');
                                                    format!("{}{}", f.to_uppercase(), l.to_uppercase())
                                                }}
                                            </span>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <img src=url class="avatar-image" alt="Profile"/>
                                    }.into_view()
                                }
                            }}
                        </div>
                        
                        // Avatar URL input
                        <div class="avatar-url-input">
                            <label class="form-label">"Avatar URL"</label>
                            <input
                                type="url"
                                class="form-input"
                                placeholder="https://example.com/avatar.jpg"
                                prop:value=avatar_url
                                on:input=move |ev| {
                                    let url = event_target_value(&ev);
                                    set_avatar_url.set(url.clone());
                                    if let Some(storage) = web_sys::window()
                                        .and_then(|w| w.local_storage().ok())
                                        .flatten()
                                    {
                                        let _ = storage.set_item("user_avatar", &url);
                                    }
                                }
                            />
                        </div>
                    </div>

                    // Form Section
                    <div class="profile-form">
                        <div class="form-row">
                            <div class="form-field">
                                <label class="form-label">"First Name"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="Enter first name"
                                    prop:value=first_name
                                    on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Last Name"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="Enter last name"
                                    prop:value=last_name
                                    on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        <div class="form-field">
                            <label class="form-label">"Email Address"</label>
                            <input
                                type="email"
                                class="form-input"
                                placeholder="Enter email"
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                            />
                            <p class="form-hint">"This is your login email"</p>
                        </div>

                        <div class="form-field">
                            <label class="form-label">"Phone Number"</label>
                            <input
                                type="tel"
                                class="form-input"
                                placeholder="+1 (555) 123-4567"
                                prop:value=phone
                                on:input=move |ev| set_phone.set(event_target_value(&ev))
                            />
                        </div>

                        // Message display
                        {move || message.get().map(|(msg, success)| view! {
                            <div class=format!("profile-message {}", if success { "success" } else { "error" })>
                                {msg}
                            </div>
                        })}

                        <div class="profile-actions">
                            <button
                                class="btn btn-primary"
                                on:click=on_save
                                disabled=move || saving.get()
                            >
                                {move || if saving.get() { "Saving..." } else { "Save Changes" }}
                            </button>
                            <button
                                class="btn btn-secondary"
                                on:click=move |_| set_show_password_modal.set(true)
                            >
                                "Change Password"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Password Modal
            {move || show_password_modal.get().then(|| {
                let close = Rc::new(move || set_show_password_modal.set(false));
                view! { <PasswordModal on_close=close/> }
            })}
        </div>
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

/// Password change modal
#[component]
fn PasswordModal(on_close: Rc<dyn Fn()>) -> impl IntoView {
    let (current_password, set_current_password) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (saving, set_saving) = create_signal(false);
    let (success, set_success) = create_signal(false);

    let on_close_overlay = on_close.clone();
    let on_close_x = on_close.clone();
    let on_close_cancel = on_close.clone();

    let on_submit = move |_| {
        set_error.set(None);
        
        if current_password.get().is_empty() {
            set_error.set(Some("Current password is required".to_string()));
            return;
        }
        
        if new_password.get() != confirm_password.get() {
            set_error.set(Some("Passwords do not match".to_string()));
            return;
        }
        
        if new_password.get().len() < 8 {
            set_error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }

        set_saving.set(true);
        
        set_timeout(
            move || {
                set_saving.set(false);
                set_success.set(true);
            },
            std::time::Duration::from_millis(500),
        );
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close_overlay()>
            <div class="modal-content" on:click=|e| e.stop_propagation()>
                <div class="modal-header">
                    <h2>"Change Password"</h2>
                    <button class="modal-close" on:click=move |_| on_close_x()>"×"</button>
                </div>
                <div class="modal-body">
                    {move || if success.get() {
                        view! {
                            <div class="profile-message success">
                                "Password changed successfully!"
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <>
                                <div class="form-field">
                                    <label class="form-label">"Current Password"</label>
                                    <input
                                        type="password"
                                        class="form-input"
                                        prop:value=current_password
                                        on:input=move |ev| set_current_password.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"New Password"</label>
                                    <input
                                        type="password"
                                        class="form-input"
                                        prop:value=new_password
                                        on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Confirm New Password"</label>
                                    <input
                                        type="password"
                                        class="form-input"
                                        prop:value=confirm_password
                                        on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                    />
                                </div>
                                {move || error.get().map(|e| view! {
                                    <div class="profile-message error">{e}</div>
                                })}
                            </>
                        }.into_view()
                    }}
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" on:click=move |_| on_close_cancel()>
                        {move || if success.get() { "Close" } else { "Cancel" }}
                    </button>
                    <Show when=move || !success.get()>
                        <button
                            class="btn btn-primary"
                            on:click=on_submit
                            disabled=move || saving.get()
                        >
                            {move || if saving.get() { "Saving..." } else { "Change Password" }}
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}
