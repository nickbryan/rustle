#![warn(clippy::all, clippy::pedantic)]

use rustle_core::ui::{Color, Rect};
use rustle_core::{Canvas, Cell, Editor, Event, Key};
use std::io;
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

#[wasm_bindgen(module = "xterm-addon-fit")]
extern "C" {
    #[wasm_bindgen(extends = TerminalAddon)]
    type FitAddon;

    #[wasm_bindgen(constructor)]
    fn new() -> FitAddon;

    #[wasm_bindgen(method)]
    fn fit(this: &FitAddon);
}

#[wasm_bindgen(module = "xterm")]
extern "C" {
    type Terminal;

    #[wasm_bindgen(constructor)]
    fn new() -> Terminal;

    #[wasm_bindgen(method)]
    fn open(this: &Terminal, parent: Element);

    #[wasm_bindgen(method)]
    fn write(this: &Terminal, data: String);

    #[wasm_bindgen(method, js_name = "attachCustomKeyEventHandler")]
    fn attach_custom_key_event_handler(
        this: &Terminal,
        handler: &Closure<dyn FnMut(KeyboardEvent) -> bool>,
    );

    #[wasm_bindgen(js_name = "ITerminalAddon")]
    type TerminalAddon;

    #[wasm_bindgen(method)]
    fn activate(this: &TerminalAddon, terminal: Terminal);

    #[wasm_bindgen(method, js_name = loadAddon)]
    fn load_addon(this: &Terminal, addon: TerminalAddon);

    #[wasm_bindgen(method, getter)]
    fn cols(this: &Terminal) -> usize;

    #[wasm_bindgen(method, getter)]
    fn rows(this: &Terminal) -> usize;

    #[wasm_bindgen(method)]
    fn focus(this: &Terminal);
}

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

impl WebCanvas {
    fn set_foreground_color(&mut self, color: Color) {
        if let Color::AnsiValue(v) = color {
            self.buffer
                .get_mut()
                .write_all(format!("\x1B[38;5;{v}m").as_bytes())
                .expect("buffer should be writable");

            return;
        }

        if let Color::Rgb(r, g, b) = color {
            self.buffer
                .get_mut()
                .write_all(format!("\x1B[38;2;{r};{g};{b}m").as_bytes())
                .expect("buffer should be writable");

            return;
        }

        self.buffer
            .get_mut()
            .write_all(format!("\x1B[{}m", color_code(color)).as_bytes())
            .expect("buffer should be writable");
    }

    fn set_background_color(&mut self, color: Color) {
        if let Color::AnsiValue(v) = color {
            self.buffer
                .get_mut()
                .write_all(format!("\x1B[48;5;{v}m").as_bytes())
                .expect("buffer should be writable");

            return;
        }

        if let Color::Rgb(r, g, b) = color {
            self.buffer
                .get_mut()
                .write_all(format!("\x1B[48;2;{r};{g};{b}m").as_bytes())
                .expect("buffer should be writable");

            return;
        }

        let mut code = color_code(color);

        if code > 0 {
            code += 10;
        }

        self.buffer
            .get_mut()
            .write_all(format!("\x1B[{code}m").as_bytes())
            .expect("buffer should be writable");
    }
}

fn color_code(color: Color) -> usize {
    match color {
        Color::Reset => 0,
        Color::Black => 30,
        Color::Red => 31,
        Color::Green => 32,
        Color::Yellow => 33,
        Color::Blue => 34,
        Color::Magenta => 35,
        Color::Cyan => 36,
        Color::Gray => 37,
        Color::DarkGray => 90,
        Color::LightRed => 91,
        Color::LightGreen => 92,
        Color::LightYellow => 93,
        Color::LightBlue => 94,
        Color::LightMagenta => 95,
        Color::LightCyan => 96,
        Color::White => 97,
        _ => unimplemented!(), // Handled above...TODO: clean this up
    }
}

impl Canvas for WebCanvas {
    fn clear(&mut self) -> anyhow::Result<(), IoError> {
        self.buffer
            .get_mut()
            .write_all("\x1B[2J".as_bytes())
            .expect("buffer should be writable");

        Ok(())
    }

    fn draw<'a, I: Iterator<Item = &'a Cell>>(&mut self, cells: I) -> anyhow::Result<(), IoError> {
        let mut prev_background = Color::Reset;
        let mut prev_foreground = Color::Reset;

        for cell in cells {
            self.position_cursor(cell.position().row, cell.position().col)?;

            if cell.background() != prev_background {
                self.set_background_color(cell.background());

                prev_background = cell.background();
            }

            if cell.foreground() != prev_foreground {
                self.set_foreground_color(cell.foreground());

                prev_foreground = cell.foreground();
            }

            self.buffer
                .get_mut()
                .write_all(cell.symbol().as_bytes())
                .expect("buffer should be writable");
        }

        self.set_background_color(Color::Reset);
        self.set_foreground_color(Color::Reset);

        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<(), IoError> {
        self.buffer
            .get_mut()
            .flush()
            .expect("fix these expectations");

        let s = String::from_utf8(self.buffer.replace(Vec::new()))
            .expect("should be able to convert buffer to string");

        self.terminal.write(s);

        Ok(())
    }

    fn hide_cursor(&mut self) -> anyhow::Result<(), IoError> {
        self.buffer
            .get_mut()
            .write_all("\x1B[?25l".as_bytes())
            .expect("buffer should be writable");

        Ok(())
    }

    fn position_cursor(&mut self, row: usize, col: usize) -> anyhow::Result<(), IoError> {
        let _x =
            u16::try_from(col).map_err(|e| IoError::new(io::ErrorKind::Other, format!("{e}")))?;
        let _y =
            u16::try_from(row).map_err(|e| IoError::new(io::ErrorKind::Other, format!("{e}")))?;

        self.buffer
            .get_mut()
            .write_all(format!("\x1B[{};{}H", row + 1, col + 1).as_bytes())
            .expect("buffer should be writable");

        Ok(())
    }

    fn show_cursor(&mut self) -> anyhow::Result<(), IoError> {
        self.buffer
            .get_mut()
            .write_all("\x1B[?25h".as_bytes())
            .expect("buffer should be writable");

        Ok(())
    }

    fn size(&self) -> anyhow::Result<Rect, IoError> {
        Ok(Rect::new(self.width, self.height))
    }
}
