//! Mobile Context - Reactive screen size detection

use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Mobile breakpoint in pixels
const MOBILE_BREAKPOINT: i32 = 768;

/// Mobile context signal
#[derive(Clone, Copy)]
pub struct MobileContext {
    pub is_mobile: ReadSignal<bool>,
}

/// Provide mobile context at app root
pub fn provide_mobile_context() {
    let (is_mobile, set_is_mobile) = create_signal(check_is_mobile());
    
    // Set up resize listener for reactive updates
    if let Some(window) = web_sys::window() {
        let closure = Closure::<dyn Fn()>::new(move || {
            set_is_mobile.set(check_is_mobile());
        });
        
        let _ = window.add_event_listener_with_callback(
            "resize",
            closure.as_ref().unchecked_ref()
        );
        
        // Leak the closure to keep it alive
        closure.forget();
    }
    
    provide_context(MobileContext { is_mobile });
}

/// Check if current screen is mobile
fn check_is_mobile() -> bool {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|w| w.as_f64())
        .map(|w| w <= MOBILE_BREAKPOINT as f64)
        .unwrap_or(false)
}

/// Use mobile context
pub fn use_mobile() -> MobileContext {
    use_context::<MobileContext>().expect("MobileContext not provided")
}

/// Convenience function to check if mobile
pub fn is_mobile() -> bool {
    use_mobile().is_mobile.get()
}
