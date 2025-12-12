//! Login page with actual authentication

use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();
        let navigate = navigate.clone();

        spawn_local(async move {
            match do_login(&email_val, &password_val).await {
                Ok(_) => {
                    // Store login state
                    if let Some(storage) = web_sys::window()
                        .and_then(|w| w.local_storage().ok())
                        .flatten()
                    {
                        let _ = storage.set_item("logged_in", "true");
                        let _ = storage.set_item("user_email", &email_val);
                    }
                    navigate("/", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-page">
            <div class="login-container">
                <h1>"SaaS Platform"</h1>
                <h2>"Sign In"</h2>
                
                <form on:submit=on_submit>
                    {move || error.get().map(|e| view! {
                        <div class="error-message">{e}</div>
                    })}
                    
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input 
                            type="email"
                            id="email"
                            placeholder="admin@demo.com"
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            prop:value=email
                            required
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input 
                            type="password"
                            id="password"
                            placeholder="Admin123!"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                            required
                        />
                    </div>
                    
                    <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                        {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>
                
                <div class="demo-credentials">
                    <p>"Demo: admin@demo.com / Admin123!"</p>
                </div>
            </div>
        </div>
    }
}

async fn do_login(email: &str, password: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "subdomain": "demo"
    });

    let mut opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&body.to_string()));

    let request = Request::new_with_str_and_init(
        "http://localhost:3000/api/v1/auth/login",
        &opts
    ).map_err(|e| format!("Request error: {:?}", e))?;

    request.headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if resp.ok() {
        Ok(())
    } else {
        Err("Invalid email or password".to_string())
    }
}
