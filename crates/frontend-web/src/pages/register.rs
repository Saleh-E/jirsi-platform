//! Register Page - New tenant registration

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use crate::api::{API_BASE, post_json};

/// Request to register a new tenant
#[derive(Debug, Clone, Serialize)]
struct RegisterTenantRequest {
    company_name: String,
    subdomain: String,
    admin_email: String,
    admin_password: String,
}

/// Response from tenant registration
#[derive(Debug, Clone, Deserialize)]
struct RegisterTenantResponse {
    success: bool,
    tenant_id: String,
    subdomain: String,
    admin_email: String,
    message: String,
}

/// Request to check subdomain availability
#[derive(Debug, Clone, Serialize)]
struct CheckSubdomainRequest {
    subdomain: String,
}

/// Response from subdomain check
#[derive(Debug, Clone, Deserialize)]
struct CheckSubdomainResponse {
    available: bool,
    subdomain: String,
    message: Option<String>,
}

/// Register Page Component - Clean, centered form for new tenant registration
#[component]
pub fn RegisterPage() -> impl IntoView {
    // Form fields
    let (company_name, set_company_name) = create_signal(String::new());
    let (subdomain, set_subdomain) = create_signal(String::new());
    let (admin_email, set_admin_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    
    // UI state
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (success, set_success) = create_signal(false);
    let (subdomain_status, set_subdomain_status) = create_signal::<Option<(bool, String)>>(None);
    let (checking_subdomain, set_checking_subdomain) = create_signal(false);
    
    // Auto-generate subdomain from company name
    let on_company_change = move |ev: web_sys::Event| {
        let value = event_target_value(&ev);
        set_company_name.set(value.clone());
        
        // Generate subdomain: lowercase, replace spaces with hyphens, remove special chars
        let generated: String = value
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '-' })
            .collect::<String>()
            .replace(' ', "-")
            .replace("--", "-");
        
        set_subdomain.set(generated);
        set_subdomain_status.set(None); // Reset status when subdomain changes
    };
    
    // Check subdomain availability
    let check_subdomain = move |_| {
        let subdomain_val = subdomain.get();
        if subdomain_val.len() < 3 {
            set_subdomain_status.set(Some((false, "Subdomain must be at least 3 characters".to_string())));
            return;
        }
        
        set_checking_subdomain.set(true);
        
        spawn_local(async move {
            let url = format!("{}/auth/check-subdomain", API_BASE);
            let req = CheckSubdomainRequest { subdomain: subdomain_val };
            
            match post_json::<_, CheckSubdomainResponse>(&url, &req).await {
                Ok(res) => {
                    let message = if res.available {
                        format!("✓ {} is available!", res.subdomain)
                    } else {
                        res.message.unwrap_or_else(|| "Not available".to_string())
                    };
                    set_subdomain_status.set(Some((res.available, message)));
                }
                Err(e) => {
                    set_subdomain_status.set(Some((false, format!("Error: {}", e))));
                }
            }
            set_checking_subdomain.set(false);
        });
    };
    
    // Submit form
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        
        // Validate
        if company_name.get().trim().is_empty() {
            set_error.set(Some("Company name is required".to_string()));
            return;
        }
        if subdomain.get().len() < 3 {
            set_error.set(Some("Subdomain must be at least 3 characters".to_string()));
            return;
        }
        if !admin_email.get().contains('@') {
            set_error.set(Some("Invalid email address".to_string()));
            return;
        }
        if password.get().len() < 8 {
            set_error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }
        if password.get() != confirm_password.get() {
            set_error.set(Some("Passwords do not match".to_string()));
            return;
        }
        
        set_loading.set(true);
        
        let req = RegisterTenantRequest {
            company_name: company_name.get(),
            subdomain: subdomain.get(),
            admin_email: admin_email.get(),
            admin_password: password.get(),
        };
        
        spawn_local(async move {
            let url = format!("{}/auth/register-tenant", API_BASE);
            
            match post_json::<_, RegisterTenantResponse>(&url, &req).await {
                Ok(res) => {
                    if res.success {
                        set_success.set(true);
                    } else {
                        set_error.set(Some("Registration failed. Please try again.".to_string()));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Registration failed: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };
    
    view! {
        <div class="register-page">
            <div class="register-container">
                // Logo/Brand
                <div class="register-header">
                    <h1 class="register-logo">"🏢 Jirsi"</h1>
                    <p class="register-tagline">"Start your Real Estate Business"</p>
                </div>
                
                // Success state
                <Show when=move || success.get()>
                    <div class="register-success">
                        <div class="success-icon">"✅"</div>
                        <h2>"Welcome Aboard!"</h2>
                        <p>"Your account has been created successfully."</p>
                        <p class="success-subdomain">
                            "Your platform is ready at: "
                            <strong>{move || format!("{}.jirsi.com", subdomain.get())}</strong>
                        </p>
                        <a href="/login" class="register-btn primary">"Go to Login →"</a>
                    </div>
                </Show>
                
                // Registration form
                <Show when=move || !success.get()>
                    <form class="register-form" on:submit=on_submit>
                        // Error message
                        {move || error.get().map(|e| view! {
                            <div class="register-error">{e}</div>
                        })}
                        
                        // Company Name
                        <div class="form-group">
                            <label for="company_name">"Company Name"</label>
                            <input 
                                type="text"
                                id="company_name"
                                placeholder="Acme Real Estate"
                                prop:value=move || company_name.get()
                                on:input=on_company_change
                                required=true
                            />
                        </div>
                        
                        // Subdomain
                        <div class="form-group">
                            <label for="subdomain">"Your Platform URL"</label>
                            <div class="subdomain-input-group">
                                <input 
                                    type="text"
                                    id="subdomain"
                                    placeholder="your-company"
                                    prop:value=move || subdomain.get()
                                    on:input=move |ev| {
                                        set_subdomain.set(event_target_value(&ev).to_lowercase());
                                        set_subdomain_status.set(None);
                                    }
                                    on:blur=check_subdomain.clone()
                                    required=true
                                />
                                <span class="subdomain-suffix">".jirsi.com"</span>
                            </div>
                            {move || subdomain_status.get().map(|(available, message)| {
                                let class = if available { "subdomain-available" } else { "subdomain-taken" };
                                view! { <span class=format!("subdomain-status {}", class)>{message}</span> }
                            })}
                            {move || checking_subdomain.get().then(|| view! {
                                <span class="subdomain-status checking">"Checking..."</span>
                            })}
                        </div>
                        
                        // Admin Email
                        <div class="form-group">
                            <label for="admin_email">"Admin Email"</label>
                            <input 
                                type="email"
                                id="admin_email"
                                placeholder="admin@yourcompany.com"
                                prop:value=move || admin_email.get()
                                on:input=move |ev| set_admin_email.set(event_target_value(&ev))
                                required=true
                            />
                        </div>
                        
                        // Password
                        <div class="form-group">
                            <label for="password">"Password"</label>
                            <input 
                                type="password"
                                id="password"
                                placeholder="At least 8 characters"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required=true
                                minlength="8"
                            />
                        </div>
                        
                        // Confirm Password
                        <div class="form-group">
                            <label for="confirm_password">"Confirm Password"</label>
                            <input 
                                type="password"
                                id="confirm_password"
                                placeholder="Repeat your password"
                                prop:value=move || confirm_password.get()
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                required=true
                            />
                        </div>
                        
                        // Submit button
                        <button 
                            type="submit" 
                            class="register-btn primary"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Creating Account..." } else { "Create My Platform" }}
                        </button>
                        
                        // Login link
                        <p class="register-login">
                            "Already have an account? "
                            <a href="/login">"Sign in"</a>
                        </p>
                    </form>
                </Show>
            </div>
        </div>
        
        // Styles
        <style>
        {r#"
.register-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #0f0f14 0%, #1a1a2e 50%, #16213e 100%);
    padding: 2rem;
}

.register-container {
    width: 100%;
    max-width: 440px;
    background: #1e1e2e;
    border-radius: 16px;
    padding: 2.5rem;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
    border: 1px solid #2a2a3a;
}

.register-header {
    text-align: center;
    margin-bottom: 2rem;
}

.register-logo {
    font-size: 2rem;
    font-weight: 700;
    color: #7c3aed;
    margin: 0 0 0.5rem 0;
}

.register-tagline {
    color: #a0a0b0;
    margin: 0;
    font-size: 1.1rem;
}

.register-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.form-group label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #d0d0e0;
}

.form-group input {
    padding: 0.875rem 1rem;
    background: #14141c;
    border: 1px solid #2a2a3a;
    border-radius: 8px;
    color: #f0f0f5;
    font-size: 1rem;
    transition: border-color 0.2s, box-shadow 0.2s;
}

.form-group input:focus {
    outline: none;
    border-color: #7c3aed;
    box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.15);
}

.form-group input::placeholder {
    color: #666;
}

.subdomain-input-group {
    display: flex;
    align-items: center;
}

.subdomain-input-group input {
    border-radius: 8px 0 0 8px;
    flex: 1;
}

.subdomain-suffix {
    padding: 0.875rem 1rem;
    background: #2a2a3a;
    border: 1px solid #2a2a3a;
    border-left: none;
    border-radius: 0 8px 8px 0;
    color: #888;
    font-size: 0.875rem;
    white-space: nowrap;
}

.subdomain-status {
    font-size: 0.8rem;
    margin-top: 0.25rem;
}

.subdomain-available { color: #22c55e; }
.subdomain-taken { color: #ef4444; }
.subdomain-status.checking { color: #a0a0b0; }

.register-error {
    padding: 0.875rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    border-radius: 8px;
    color: #ef4444;
    font-size: 0.875rem;
}

.register-btn {
    padding: 1rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    text-decoration: none;
    text-align: center;
}

.register-btn.primary {
    background: linear-gradient(135deg, #7c3aed 0%, #6366f1 100%);
    color: white;
}

.register-btn.primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 20px rgba(124, 58, 237, 0.4);
}

.register-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
}

.register-login {
    text-align: center;
    color: #888;
    font-size: 0.875rem;
    margin: 0;
}

.register-login a {
    color: #7c3aed;
    text-decoration: none;
    font-weight: 500;
}

.register-login a:hover {
    text-decoration: underline;
}

.register-success {
    text-align: center;
    padding: 2rem 0;
}

.success-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
}

.register-success h2 {
    color: #f0f0f5;
    margin: 0 0 0.5rem 0;
}

.register-success p {
    color: #a0a0b0;
    margin: 0 0 1.5rem 0;
}

.success-subdomain {
    background: #14141c;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1.5rem !important;
}

.success-subdomain strong {
    color: #7c3aed;
}
        "#}
        </style>
    }
}
