#![recursion_limit = "512"]

pub mod app;

#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod admin_session;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::mount::hydrate_body;

    console_error_panic_hook::set_once();

    hydrate_body(App);
}
