//! PWA Install Prompt Component
//!
//! Shows an install banner when the app is running as a website
//! and the browser supports PWA installation.

use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// PWA Install Prompt Component
/// 
/// Displays a banner prompting users to install the PWA on mobile devices.
/// Uses JsValue to handle the beforeinstallprompt event since it's not in web_sys.
#[component]
pub fn PwaInstallPrompt() -> impl IntoView {
    let (show_banner, set_show_banner) = create_signal(false);
    let (deferred_prompt, set_deferred_prompt) = create_signal::<Option<JsValue>>(None);
    let (is_installed, set_is_installed) = create_signal(false);

    // Check if already installed
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            // Check display-mode media query
            if let Ok(media) = window.match_media("(display-mode: standalone)") {
                if let Some(m) = media {
                    if m.matches() {
                        set_is_installed.set(true);
                        return;
                    }
                }
            }
            
            // Check if running in iOS Safari (uses navigator.standalone)
            if let Ok(standalone) = js_sys::Reflect::get(&window.navigator(), &"standalone".into()) {
                if standalone.as_bool().unwrap_or(false) {
                    set_is_installed.set(true);
                    return;
                }
            }
        }
    });

    // Listen for beforeinstallprompt event
    create_effect(move |_| {
        if is_installed.get() {
            return;
        }

        if let Some(window) = web_sys::window() {
            let set_prompt = set_deferred_prompt;
            let set_banner = set_show_banner;
            
            let handler = Closure::wrap(Box::new(move |e: web_sys::Event| {
                e.prevent_default();
                
                // Store the event as JsValue for later use
                let event: JsValue = e.into();
                set_prompt.set(Some(event));
                
                // Check if user has dismissed before
                if !was_dismissed() {
                    set_banner.set(true);
                }
            }) as Box<dyn Fn(web_sys::Event)>);
            
            let _ = window.add_event_listener_with_callback(
                "beforeinstallprompt",
                handler.as_ref().unchecked_ref(),
            );
            handler.forget();
        }
    });

    // Install handler
    let handle_install = move |_| {
        if let Some(prompt) = deferred_prompt.get() {
            // Call prompt() method on the event
            if let Ok(prompt_fn) = js_sys::Reflect::get(&prompt, &"prompt".into()) {
                if let Some(func) = prompt_fn.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(&prompt);
                }
            }
            
            // We can't easily await user_choice in this simplified version
            // Just hide the banner after install attempt
            set_show_banner.set(false);
        }
    };

    // Dismiss handler
    let handle_dismiss = move |_| {
        set_show_banner.set(false);
        save_dismissed();
    };

    view! {
        <Show when=move || show_banner.get() && !is_installed.get()>
            <div class="pwa-install-banner">
                <div class="pwa-install-content">
                    <span class="pwa-install-icon">"📱"</span>
                    <div class="pwa-install-text">
                        <strong>"Install Jirsi"</strong>
                        <span class="pwa-install-desc">"Get the full app experience"</span>
                    </div>
                </div>
                <div class="pwa-install-actions">
                    <button class="btn-pwa-install" on:click=handle_install>
                        "Install"
                    </button>
                    <button class="btn-pwa-dismiss" on:click=handle_dismiss>
                        "Not now"
                    </button>
                </div>
            </div>
        </Show>
    }
}

/// Check if user has dismissed the prompt before
fn was_dismissed() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("pwa_install_dismissed").ok())
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Save dismissed state
fn save_dismissed() {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("pwa_install_dismissed", "true");
    }
}

/// iOS install instructions component (iOS doesn't support beforeinstallprompt)
#[component]
pub fn IosInstallInstructions() -> impl IntoView {
    let (is_ios, set_is_ios) = create_signal(false);
    let (show_instructions, set_show_instructions) = create_signal(false);

    // Detect iOS Safari
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let user_agent = navigator.user_agent().unwrap_or_default().to_lowercase();
            
            let is_ios_device = user_agent.contains("iphone") || 
                               user_agent.contains("ipad") || 
                               user_agent.contains("ipod");
            
            let is_safari = user_agent.contains("safari") && 
                           !user_agent.contains("chrome") &&
                           !user_agent.contains("crios");
            
            // Check if running in standalone mode
            if let Ok(standalone) = js_sys::Reflect::get(&navigator, &"standalone".into()) {
                if standalone.as_bool().unwrap_or(false) {
                    // Already installed
                    return;
                }
            }
            
            if is_ios_device && is_safari && !was_dismissed() {
                set_is_ios.set(true);
                set_show_instructions.set(true);
            }
        }
    });

    let handle_dismiss = move |_| {
        set_show_instructions.set(false);
        save_dismissed();
    };

    view! {
        <Show when=move || is_ios.get() && show_instructions.get()>
            <div class="ios-install-banner">
                <div class="ios-install-content">
                    <p><strong>"Install Jirsi on your iPhone"</strong></p>
                    <ol class="ios-install-steps">
                        <li>"Tap the Share button " <span class="ios-icon">"⬆️"</span></li>
                        <li>"Scroll down and tap \"Add to Home Screen\""</li>
                        <li>"Tap \"Add\" to install"</li>
                    </ol>
                </div>
                <button class="btn-ios-dismiss" on:click=handle_dismiss>
                    "Got it"
                </button>
            </div>
        </Show>
    }
}
