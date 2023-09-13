#![warn(clippy::all, clippy::pedantic)]

use crate::{
    ui::WebCanvas,
    xterm::{FitAddon, Terminal},
};
use rustle_core::{Editor, Event, Key};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;

mod ui;
mod xterm;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
/// # Errors
/// TODO...
pub fn main_js() -> Result<(), JsValue> {
    // TODO: convert the errors in here to anyhow

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
        if event.type_() != "keydown" {
            return true;
        }

        // TODO: handle error
        // TODO: Should this be a normal send (async)
        let _ = tx.blocking_send(Event::KeyPressed(match event.key().as_str() {
            "Enter" => Key::Enter,
            "ArrowLeft" => Key::Left,
            "ArrowUp" => Key::Up,
            "ArrowRight" => Key::Right,
            "ArrowDown" => Key::Down,
            "Tab" => Key::Tab,
            "Backspace" => Key::Backspace,
            "Escape" => Key::Esc,
            "Insert" => Key::Insert,
            "Delete" => Key::Delete,
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            key => {
                // TODO: clean this up
                if key.len() == 1 {
                    if event.ctrl_key() {
                        Key::Ctrl(key.chars().next().unwrap())
                    } else {
                        Key::Char(key.chars().next().unwrap())
                    }
                } else {
                    Key::Unknown
                }
            }
        }));

        true
    });

    let fit = FitAddon::new();

    terminal.attach_custom_key_event_handler(&c);
    terminal.load_addon(fit.clone().into());
    fit.fit();

    c.forget();

    terminal.focus();

    spawn_local(async move {
        let mut canvas = WebCanvas::new(terminal.cols(), terminal.rows(), terminal);

        let mut editor = Editor::new(&mut canvas).expect("creating editor");

        editor
            .consume(Box::pin(ReceiverStream::new(rx)))
            .await
            .expect("consuming event stream");
    });

    Ok(())
}
