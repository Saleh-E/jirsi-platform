//! Theme Context - Global theme state with hot-swappable CSS variables
//!
//! Provides theme management (Light/Dark/System) and brand color customization

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System, // Auto-detect from OS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: ThemeMode,
    pub brand_hue: u16,        // 0-360 for HSL
    pub brand_saturation: u8,  // 0-100
    pub brand_lightness: u8,   // 0-100
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            brand_hue: 217,         // Blue
            brand_saturation: 91,
            brand_lightness: 60,
        }
    }
}

impl Theme {
    pub fn apply_to_dom(&self) {
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");
        let html = document.document_element().expect("no html element");
        
        // Set theme mode data attribute
        let mode_str = match self.mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::System => {
                // Detect system preference
                let prefers_dark = window
                    .match_media("(prefers-color-scheme: dark)")
                    .ok()
                    .flatten()
                    .map(|mql| mql.matches())
                    .unwrap_or(false);
                if prefers_dark { "dark" } else { "light" }
            }
        };
        html.set_attribute("data-theme", mode_str).ok();
        
        // Inject brand color CSS variables using setAttribute
        let existing_style = html.get_attribute("style").unwrap_or_default();
        let prefix = if existing_style.is_empty() { 
            String::new() 
        } else { 
            format!("{}; ", existing_style)
        };
        let new_style = format!(
            "{}--brand-hue: {}; --brand-saturation: {}%; --brand-lightness: {}%;",
            prefix,
            self.brand_hue, 
            self.brand_saturation, 
            self.brand_lightness
        );
        html.set_attribute("style", &new_style).ok();
    }
}

#[derive(Clone, Copy)]
pub struct ThemeContext(pub RwSignal<Theme>);

pub fn provide_theme_context() {
    let theme = create_rw_signal(Theme::default());
    provide_context(ThemeContext(theme));
    
    // Apply theme on mount and when it changes
    create_effect(move |_| {
        theme.get().apply_to_dom();
    });
}

pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().expect("ThemeContext not provided")
}
