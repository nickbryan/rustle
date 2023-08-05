use crate::xterm::Terminal;
use rustle_core::ui::{Color, Rect};
use rustle_core::{Canvas, Cell};
use std::io;
use std::io::{Error as IoError, Write};

pub(crate) struct WebCanvas {
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
