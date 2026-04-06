#![recursion_limit = "512"]

pub mod app;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::mount::hydrate_body;

    console_error_panic_hook::set_once();

    hydrate_body(App);
}
