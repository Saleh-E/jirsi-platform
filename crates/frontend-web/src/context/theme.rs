//! Theme Context - Dark Mode management with persistence

use leptos::*;

/// Theme context holding the dark mode state
#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub is_dark: ReadSignal<bool>,
    pub set_is_dark: WriteSignal<bool>,
}

impl ThemeContext {
    /// Toggle between dark and light mode
    pub fn toggle(&self) {
        let new_value = !self.is_dark.get();
        self.set_is_dark.set(new_value);
        Self::apply_theme(new_value);
        Self::save_to_storage(new_value);
    }

    /// Apply theme to DOM
    fn apply_theme(is_dark: bool) {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = document.body() {
                let class_list = body.class_list();
                if is_dark {
                    let _ = class_list.add_1("dark");
                    let _ = class_list.remove_1("light");
                } else {
                    let _ = class_list.add_1("light");
                    let _ = class_list.remove_1("dark");
                }
            }
        }
    }

    /// Save preference to localStorage
    fn save_to_storage(is_dark: bool) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("theme", if is_dark { "dark" } else { "light" });
        }
    }

    /// Load preference from localStorage (default to dark mode)
    fn load_from_storage() -> bool {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            if let Ok(Some(theme)) = storage.get_item("theme") {
                return theme == "dark";
            }
        }
        true // Default to dark mode
    }
}

/// Provide theme context at app root
pub fn provide_theme_context() {
    let initial = ThemeContext::load_from_storage();
    let (is_dark, set_is_dark) = create_signal(initial);
    
    // Apply initial theme
    ThemeContext::apply_theme(initial);
    
    let ctx = ThemeContext { is_dark, set_is_dark };
    provide_context(ctx);
}

/// Use theme context from any component
pub fn use_theme() -> ThemeContext {
    expect_context::<ThemeContext>()
}

/// Theme toggle button component
#[component]
pub fn ThemeToggle() -> impl IntoView {
    let theme = use_theme();
    
    view! {
        <button
            class="theme-toggle"
            on:click=move |_| theme.toggle()
            title=move || if theme.is_dark.get() { "Switch to Light Mode" } else { "Switch to Dark Mode" }
        >
            {move || if theme.is_dark.get() {
                view! { <span class="theme-icon">"☀️"</span> }.into_view()
            } else {
                view! { <span class="theme-icon">"🌙"</span> }.into_view()
            }}
        </button>
    }
}
