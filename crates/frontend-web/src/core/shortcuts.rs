//! Ghost Keys: Global Keyboard Shortcuts
//! Provides "God Mode" navigation without touching the mouse.

use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

/// Component that initializes global shortcuts (must be inside Router)
#[component]
pub fn ShortcutInitializer() -> impl IntoView {
    // Initialize Ghost Keys (g+d, g+c, n+c, etc.)
    provide_global_shortcuts();
    view! {}
}

/// Provide global keyboard shortcuts at app root
pub fn provide_global_shortcuts() {
    let navigate = use_navigate();
    let last_key = create_rw_signal(0f64);
    let key_sequence = create_rw_signal(String::new());

    window_event_listener(ev::keydown, move |ev| {
        // Ignore if typing in an input/textarea
        if let Some(target) = ev.target() {
            if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                let tag = el.tag_name().to_uppercase();
                if tag == "INPUT" || tag == "TEXTAREA" || el.is_content_editable() {
                    return;
                }
            }
        }

        let now = js_sys::Date::now();
        
        // Reset sequence if more than 500ms passed
        if now - last_key.get() > 500.0 {
            key_sequence.set(String::new());
        }
        last_key.set(now);

        // Build the key sequence
        key_sequence.update(|s| s.push_str(&ev.key().to_lowercase()));
        let seq = key_sequence.get();

        // The "God Mode" Shortcuts
        let nav = navigate.clone();
        match seq.as_str() {
            // Navigation: g + letter
            "gd" => {
                nav("/app/dashboard", Default::default());
                key_sequence.set(String::new());
            }
            "gc" => {
                nav("/app/crm/entity/contact", Default::default());
                key_sequence.set(String::new());
            }
            "gp" => {
                nav("/app/crm/entity/property", Default::default());
                key_sequence.set(String::new());
            }
            "gx" => {
                nav("/app/crm/entity/deal", Default::default());
                key_sequence.set(String::new());
            }
            "gi" => {
                nav("/app/inbox", Default::default());
                key_sequence.set(String::new());
            }
            "gt" => {
                nav("/app/tasks", Default::default());
                key_sequence.set(String::new());
            }
            
            // Quick Actions: n + letter
            "nc" => {
                nav("/app/crm/entity/contact/new", Default::default());
                key_sequence.set(String::new());
            }
            "np" => {
                nav("/app/crm/entity/property/new", Default::default());
                key_sequence.set(String::new());
            }
            "nd" => {
                nav("/app/crm/entity/deal/new", Default::default());
                key_sequence.set(String::new());
            }
            
            _ => {}
        }
    });
}
