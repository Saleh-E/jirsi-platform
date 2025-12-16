//! Frontend Web - Leptos WebAssembly Application
//!
//! Generic UI shell for the SaaS platform.

pub mod api;
pub mod app;
pub mod components;
pub mod context;
pub mod layouts;
pub mod models;
pub mod pages;

pub use app::App;

use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

/// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| {
        // Provide contexts at app root
        context::provide_theme_context();
        context::provide_mobile_context();
        view! { <App/> }
    });
}
