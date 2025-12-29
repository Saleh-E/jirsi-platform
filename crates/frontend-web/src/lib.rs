//! Frontend Web - Leptos WebAssembly Application
//!
//! Generic UI shell for the SaaS platform.

// Cinematic OS Modules
pub mod core;
pub mod design_system;
pub mod widgets;
pub mod features;

// Legacy Modules
pub mod api;
pub mod app;
pub mod components;
pub mod context;
pub mod layouts;
pub mod models;
pub mod offline;
pub mod pages;
pub mod utils;

pub use app::App;

use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::core::sync_engine::provide_sync_engine;
use crate::design_system::feedback::toast::provide_toast_context;

/// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| {
        // Provide Cinematic OS Contexts
        provide_sync_engine();
        provide_toast_context();
        
        // Provide Legacy Contexts
        context::provide_jirsi_theme();
        context::provide_mobile_context();
        context::network_status::provide_network_status();
        
        view! { <App/> }
    });
}
