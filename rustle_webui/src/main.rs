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

/// A `Runtime` implementation for the WebAssembly environment.
/// It uses `wasm_bindgen_futures::spawn_local` to spawn tasks.
struct WasmRuntime;

impl Runtime for WasmRuntime {
    /// Spawns a new future on the current task.
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}

fn main() -> Result<()> {
    // Set up a panic hook to display panic messages in the browser's console.
    console_error_panic_hook::set_once();

    // Create a new xterm.js terminal instance.
    let terminal = Terminal::new();

    // Get the terminal DOM element and open the terminal in it.
    let terminal_elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("terminal")
        .context("getting terminal element")?;
    terminal.open(terminal_elem);

    // Create a channel to send keyboard events from the browser to the editor.
    let (tx, rx) = mpsc::channel(1);

    // Create a closure to handle keyboard events from the browser.
    // This closure captures the sender of the channel and sends key events to it.
    let c = Closure::new(move |event: KeyboardEvent| {
        if event.type_() != "keydown" {
            return true;
        }

        // Map the JavaScript KeyboardEvent to the editor's internal `Key` type.
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
            web_sys::console::log_1(&format!("Failed to send key event: {e}").into());
        }

        true
    });

    // Attach the keyboard event handler to the terminal.
    terminal.attach_custom_key_event_handler(&c);

    // Load and use the "fit" addon to make the terminal fit the viewport.
    let fit = FitTerminalAddon::new();
    terminal.load_addon(fit.clone().into());
    fit.fit();

    // Forget the closure to prevent it from being deallocated.
    c.forget();

    // Focus the terminal to start capturing keyboard events.
    terminal.focus();

    // Spawn the editor task.
    // This task will run the editor's event loop, consuming the event stream from the channel.
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
