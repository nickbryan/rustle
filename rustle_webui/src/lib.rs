#![warn(clippy::all, clippy::pedantic)]

use rustle_core::{Editor, Event, Key};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, KeyboardEvent};

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(module = "xterm")]
extern "C" {
    type Terminal;

    #[wasm_bindgen(constructor)]
    fn new() -> Terminal;

    #[wasm_bindgen(method)]
    fn open(this: &Terminal, parent: Element);

    #[wasm_bindgen(method)]
    pub fn write(this: &Terminal, data: String);

    #[wasm_bindgen(method, js_name = attachCustomKeyEventHandler)]
    pub fn attach_custom_key_event_handler(
        this: &Terminal,
        handler: &Closure<dyn FnMut(KeyboardEvent) -> bool>,
    );
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let terminal = Terminal::new();

    let terminal_elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("terminal")
        .unwrap();

    terminal.open(terminal_elem);

    let (tx, rx) = mpsc::channel(1);

    let c = Closure::new(move |event: KeyboardEvent| {
        let _ = tx.blocking_send(Event::KeyPressed(Key::Char(
            event.key().chars().next().unwrap(),
        ))); // Should this be a normal send (async);
        true
    });

    terminal.attach_custom_key_event_handler(&c);

    c.forget();

    terminal.write(String::from("Hellossss!"));

    let mut editor = Editor::new();
    spawn_local(async move {
        editor
            .consume(Box::pin(ReceiverStream::new(rx)))
            .await
            .expect("stuf and thath");
    });

    Ok(())
}
