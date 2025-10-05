#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use rustle_core::{Editor, Event, Key};
use rustle_state::Runtime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use wasm_bindgen::prelude::Closure;
use web_sys::KeyboardEvent;

use crate::{
    backend::WebCanvas,
    xterm::{FitTerminalAddon, Terminal},
};

mod backend;
mod xterm;

struct WasmRuntime;

impl Runtime for WasmRuntime {
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}

fn main() -> Result<()> {
    console_error_panic_hook::set_once();

    let terminal = Terminal::new();

    let terminal_elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("terminal")
        .context("getting terminal element")?;

    terminal.open(terminal_elem);

    let (tx, rx) = mpsc::channel(1);

    let c = Closure::new(move |event: KeyboardEvent| {
        if event.type_() != "keydown" {
            return true;
        }

        let send_result = tx.try_send(Event::KeyPressed(match event.key().as_str() {
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
                if key.len() == 1 {
                    if let Some(c) = key.chars().next() {
                        if event.ctrl_key() {
                            Key::Ctrl(c)
                        } else {
                            Key::Char(c)
                        }
                    } else {
                        Key::Unknown
                    }
                } else {
                    Key::Unknown
                }
            }
        }));

        if let Err(e) = send_result {
            web_sys::console::log_1(&format!("Failed to send key event: {}", e).into());
        }

        true
    });

    terminal.attach_custom_key_event_handler(&c);

    let fit = FitTerminalAddon::new();
    terminal.load_addon(fit.clone().into());
    fit.fit();

    c.forget();

    terminal.focus();

    wasm_bindgen_futures::spawn_local(async move {
        Editor::new(
            WebCanvas::new(terminal.cols(), terminal.rows(), terminal),
            &WasmRuntime,
        )
        .consume(Box::pin(ReceiverStream::new(rx)))
        .await
        .expect("consuming event stream");
    });

    Ok(())
}
