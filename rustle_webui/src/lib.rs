use wasm_bindgen::prelude::*;
use web_sys::Element;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(module = "xterm")]
extern "C" {
    type Terminal;

    #[wasm_bindgen(constructor)]
    fn new() -> Terminal;

    #[wasm_bindgen(method)]
    fn open(this: &Terminal, parent: Element);

    #[wasm_bindgen(method)]
    pub fn write(this: &Terminal, data: String);
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let terminal = Terminal::new();

    terminal.open(
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("terminal")
            .unwrap(),
    );

    terminal.write(String::from("Hello World, the deploy works!"));

    Ok(())
}
