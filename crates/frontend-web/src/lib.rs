//! Frontend Web - Leptos WebAssembly Application
//!
//! Generic UI shell for the SaaS platform.

pub mod api;
pub mod app;
pub mod components;
pub mod models;
pub mod pages;

pub use app::App;

use wasm_bindgen::prelude::wasm_bindgen;

/// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
