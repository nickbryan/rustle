#![warn(clippy::all, clippy::pedantic)]

use rustle_core::ui::Rect;
use rustle_core::{Canvas, Cell, Editor, Event, Key};
use std::io::{Error as IoError, Write};
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

    let width = terminal_elem.client_width();
    let height = terminal_elem.client_height();

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

    spawn_local(async move {
        let mut canvas = WebCanvas::new(width as usize, height as usize, terminal);

        let mut editor = Editor::new(&mut canvas).expect("creating editor");

        editor
            .consume(Box::pin(ReceiverStream::new(rx)))
            .await
            .expect("consuming event stream");
    });

    Ok(())
}

struct WebCanvas {
    width: usize,
    height: usize,
    terminal: Terminal,
    buffer: std::cell::Cell<Vec<u8>>,
}

impl WebCanvas {
    pub fn new(width: usize, height: usize, terminal: Terminal) -> Self {
        // TODO: type the size as rect or something?
        Self {
            width,
            height,
            terminal,
            buffer: std::cell::Cell::new(Vec::new()),
        }
    }
}

impl Canvas for WebCanvas {
    fn clear(&mut self) -> anyhow::Result<(), IoError> {
        Ok(())
    }

    fn draw<'a, I: Iterator<Item = &'a Cell>>(&mut self, cells: I) -> anyhow::Result<(), IoError> {
        for cell in cells {
            self.buffer
                .get_mut()
                .write(cell.symbol().as_bytes())
                .expect("buffer should be writable");
        }

        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<(), IoError> {
        self.buffer
            .get_mut()
            .flush()
            .expect("fix these epectations");

        let s = String::from_utf8(self.buffer.replace(Vec::new()))
            .expect("should be able to convert buffer to string");

        self.terminal.write(s);
        Ok(())
    }

    fn hide_cursor(&mut self) -> anyhow::Result<(), IoError> {
        Ok(())
    }

    fn position_cursor(&mut self, row: usize, col: usize) -> anyhow::Result<(), IoError> {
        Ok(())
    }

    fn show_cursor(&mut self) -> anyhow::Result<(), IoError> {
        Ok(())
    }

    fn size(&self) -> anyhow::Result<Rect, IoError> {
        Ok(Rect::new(self.width, self.height))
    }
}
