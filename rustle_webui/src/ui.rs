use crate::xterm::Terminal;
use anyhow::{Context, Result};
use rustle_core::ui::{
    render::{Canvas, Cell},
    values::{Color, Rect},
};
use std::{
    io::{Error as IoError, Write},
    mem,
};

pub(crate) struct WebCanvas {
    width: u16,
    height: u16,
    terminal: Terminal,
    buffer: Vec<u8>,
}

impl WebCanvas {
    pub(crate) fn new(width: u16, height: u16, terminal: Terminal) -> Self {
        Self {
            width,
            height,
            terminal,
            buffer: Vec::new(),
        }
    }
}

impl WebCanvas {
    fn set_foreground_color(&mut self, color: Color) -> Result<()> {
        if let Color::AnsiValue(v) = color {
            self.buffer
                .write_all(format!("\x1B[38;5;{v}m").as_bytes())
                .context("writing ansi color value to buffer")?;

            return Ok(());
        }

        if let Color::Rgb(r, g, b) = color {
            self.buffer
                .write_all(format!("\x1B[38;2;{r};{g};{b}m").as_bytes())
                .expect("writing rgb color value to buffer");

            return Ok(());
        }

        self.buffer
            .write_all(format!("\x1B[{}m", color_code(color)).as_bytes())
            .context("writing color code color value to buffer")
    }

    fn set_background_color(&mut self, color: Color) -> Result<()> {
        if let Color::AnsiValue(v) = color {
            self.buffer
                .write_all(format!("\x1B[48;5;{v}m").as_bytes())
                .context("writing ansi color value to buffer")?;

            return Ok(());
        }

        if let Color::Rgb(r, g, b) = color {
            self.buffer
                .write_all(format!("\x1B[48;2;{r};{g};{b}m").as_bytes())
                .context("writing rgb color value to buffer")?;

            return Ok(());
        }

        let mut code = color_code(color);
        if code > 0 {
            code += 10;
        }

        self.buffer
            .write_all(format!("\x1B[{code}m").as_bytes())
            .context("writing color code color value to buffer")
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
        _ => unreachable!(), // Expected to be handled before this. // TODO: clean this up
    }
}

impl Canvas for WebCanvas {
    fn clear(&mut self) -> Result<(), IoError> {
        self.buffer
            .write_all("\x1B[2J".as_bytes())
            .context("writing clear to buffer")
            .map_err(|e| IoError::other(format!("{e}")))
    }

    fn draw<'a, I: Iterator<Item = &'a Cell>>(&mut self, cells: I) -> Result<(), IoError> {
        let mut prev_background = Color::Reset;
        let mut prev_foreground = Color::Reset;

        for cell in cells {
            self.position_cursor(cell.position().row, cell.position().col)?;

            if cell.background() != prev_background {
                self.set_background_color(cell.background())
                    .context("setting background color")
                    .map_err(|e| IoError::other(format!("{e}")))?;

                prev_background = cell.background();
            }

            if cell.foreground() != prev_foreground {
                self.set_foreground_color(cell.foreground())
                    .context("setting foreground color")
                    .map_err(|e| IoError::other(format!("{e}")))?;

                prev_foreground = cell.foreground();
            }

            self.buffer
                .write_all(cell.symbol().as_bytes())
                .context("writing cell symbol to buffer")
                .map_err(|e| IoError::other(format!("{e}")))?;
        }

        self.set_background_color(Color::Reset)
            .context("setting background color")
            .map_err(|e| IoError::other(format!("{e}")))?;
        self.set_foreground_color(Color::Reset)
            .context("setting foreground color")
            .map_err(|e| IoError::other(format!("{e}")))?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.buffer
            .flush()
            .context("flushing buffer")
            .map_err(|e| IoError::other(format!("{e}")))?;

        let s = String::from_utf8(mem::take(&mut self.buffer))
            .context("converting buffer tto string")
            .map_err(|e| IoError::other(format!("{e}")))?;

        self.terminal.write(s);

        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), IoError> {
        self.buffer
            .write_all("\x1B[?25l".as_bytes())
            .context("writing hide cursor to buffer")
            .map_err(|e| IoError::other(format!("{e}")))
    }

    fn position_cursor(&mut self, row: u16, col: u16) -> Result<(), IoError> {
        self.buffer
            .write_all(format!("\x1B[{};{}H", row + 1, col + 1).as_bytes())
            .context("writing position cursor to buffer")
            .map_err(|e| IoError::other(format!("{e}")))
    }

    fn show_cursor(&mut self) -> Result<(), IoError> {
        self.buffer
            .write_all("\x1B[?25h".as_bytes())
            .context("writing show cursor to buffer")
            .map_err(|e| IoError::other(format!("{e}")))
    }

    fn size(&self) -> Result<Rect, IoError> {
        Ok(Rect::new(self.width, self.height))
    }
}
