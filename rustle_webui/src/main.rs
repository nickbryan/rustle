#![warn(clippy::all, clippy::pedantic)]

use crate::{
    ui::WebCanvas,
    xterm::{FitTerminalAddon, Terminal},
};
use rustle_core::{Editor, Event, Key};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use wasm_bindgen::prelude::{Closure, JsValue};
use web_sys::KeyboardEvent;

mod ui;
mod xterm;

fn main() -> Result<(), JsValue> {
    // TODO: find a better way to maintain encapsulation with state and actor - do we inject a runtime trait somehow?
    // TODO: can we stop editor.rs State and Action from needing to be public?
    // TODO: review and check if the tui crate needs any refactoring to make it more idiomatic
    // TODO: convert the errors in here to anyhow
    // TODO: check if anyhow is the right choice to use throughout both ui crates of if we need to use a combination fo thiserror and anyhow (as some of it is kind of a lib)

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

    terminal.attach_custom_key_event_handler(&c);

    let fit = FitTerminalAddon::new();
    terminal.load_addon(fit.clone().into());
    fit.fit();

    c.forget();

    terminal.focus();

    wasm_bindgen_futures::spawn_local(async move {
        let canvas = WebCanvas::new(terminal.cols(), terminal.rows(), terminal);

        let (mut editor, mut actor) = Editor::new(canvas);

        wasm_bindgen_futures::spawn_local(async move { actor.act().await });

        editor
            .consume(Box::pin(ReceiverStream::new(rx)))
            .await
            .expect("consuming event stream");
    });

    Ok(())
}
