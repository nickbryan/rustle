use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers},
    style::{Color as CrosstermColor, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rustle_core::{
    ui::{
        render::{Canvas, Cell},
        values::{Color as RustleColor, Rect},
    },
    Event, EventStream, Key as CoreKey,
};
use std::io::{Error as IoError, Write};

/// Newtype to allow mapping RustleColor to CrosstermColor.
struct Color(RustleColor);

/// Canvas implementation for crossterm.
pub struct CrosstermCanvas<W: Write> {
    out: W,
}

impl<W: Write> CrosstermCanvas<W> {
    /// Creates a new CrosstermCanvas.
    pub fn new(mut out: W) -> Result<Self, IoError> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(out, EnterAlternateScreen)?;
        crossterm::execute!(out, EnableMouseCapture)?;

        Ok(Self { out })
    }
}

impl<W: Write> Drop for CrosstermCanvas<W> {
    /// Ensures that we LeaveAlternateScreen and disable_raw_mode before the application ends to
    /// return the user terminal back to normal.
    fn drop(&mut self) {
        crossterm::execute!(self.out, DisableMouseCapture)
            .expect("should be able to execute disable mouse capture command");
        crossterm::execute!(self.out, LeaveAlternateScreen)
            .expect("should be able to execute leave alternate screen command");
        crossterm::terminal::disable_raw_mode()
            .expect("should be able to execute disable raw mode command");
    }
}

impl<W: Write> Canvas for CrosstermCanvas<W> {
    fn clear(&mut self) -> Result<(), IoError> {
        crossterm::queue!(self.out, Clear(ClearType::All))
    }

    fn draw<'a, I>(&mut self, cells: I) -> Result<(), IoError>
    where
        I: Iterator<Item = &'a Cell>,
    {
        let mut prev_background = Color(RustleColor::Reset);
        let mut prev_foreground = Color(RustleColor::Reset);

        for cell in cells {
            self.position_cursor(cell.position().row, cell.position().col)?;

            if cell.background() != prev_background.0 {
                crossterm::queue!(
                    self.out,
                    SetBackgroundColor(CrosstermColor::from(Color(cell.background())))
                )?;

                prev_background = Color(cell.background());
            }

            if cell.foreground() != prev_foreground.0 {
                crossterm::queue!(
                    self.out,
                    SetForegroundColor(CrosstermColor::from(Color(cell.foreground())))
                )?;

                prev_foreground = Color(cell.foreground());
            }

            crossterm::queue!(self.out, Print(cell.symbol()))?;
        }

        crossterm::queue!(
            self.out,
            SetBackgroundColor(CrosstermColor::from(Color(RustleColor::Reset))),
            SetForegroundColor(CrosstermColor::from(Color(RustleColor::Reset))),
        )
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.out.flush()
    }

    fn hide_cursor(&mut self) -> Result<(), IoError> {
        crossterm::queue!(self.out, Hide)
    }

    fn position_cursor(&mut self, row: u16, col: u16) -> Result<(), IoError> {
        crossterm::queue!(self.out, MoveTo(col, row))
    }

    fn show_cursor(&mut self) -> Result<(), IoError> {
        crossterm::queue!(self.out, Show)
    }

    fn size(&self) -> Result<Rect, IoError> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Rect::new(width, height))
    }
}

impl From<Color> for CrosstermColor {
    fn from(color: Color) -> Self {
        match color.0 {
            RustleColor::Reset => CrosstermColor::Reset,
            RustleColor::Black => CrosstermColor::Black,
            RustleColor::Red => CrosstermColor::DarkRed,
            RustleColor::Green => CrosstermColor::DarkGreen,
            RustleColor::Yellow => CrosstermColor::DarkYellow,
            RustleColor::Blue => CrosstermColor::DarkBlue,
            RustleColor::Magenta => CrosstermColor::DarkMagenta,
            RustleColor::Cyan => CrosstermColor::DarkCyan,
            RustleColor::Gray => CrosstermColor::Grey,
            RustleColor::DarkGray => CrosstermColor::DarkGrey,
            RustleColor::LightRed => CrosstermColor::Red,
            RustleColor::LightGreen => CrosstermColor::Green,
            RustleColor::LightBlue => CrosstermColor::Blue,
            RustleColor::LightYellow => CrosstermColor::Yellow,
            RustleColor::LightMagenta => CrosstermColor::Magenta,
            RustleColor::LightCyan => CrosstermColor::Cyan,
            RustleColor::White => CrosstermColor::White,
            RustleColor::AnsiValue(v) => CrosstermColor::AnsiValue(v),
            RustleColor::Rgb(r, g, b) => CrosstermColor::Rgb { r, g, b },
        }
    }
}

/// Newtype to allow mapping crossterm::event::KeyEvent to VelmKey.
struct Key(CoreKey);

/// Map the events coming from the crossterm EventStream into the events that are expected by the application.
pub fn map_crossterm_event_stream() -> EventStream {
    use futures::StreamExt;

    Box::pin(crossterm::event::EventStream::new().map(|possible_event| {
        use crossterm::event as ctevent;

        match possible_event {
            Ok(ctevent::Event::Key(key)) => Event::KeyPressed(Key::from(key).0),
            Ok(crossterm::event::Event::FocusGained)
            | Ok(crossterm::event::Event::FocusLost)
            | Ok(crossterm::event::Event::Paste(_))
            | Ok(ctevent::Event::Mouse(_)) => Event::MouseInputReceived,
            Ok(ctevent::Event::Resize(x, y)) => Event::WindowResized(x, y),
            Err(e) => Event::ReadFailed(e),
        }
    }))
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        match event {
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Key(CoreKey::Enter),
            KeyEvent {
                code: KeyCode::Tab, ..
            } => Key(CoreKey::Tab),
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Key(CoreKey::Backspace),
            KeyEvent {
                code: KeyCode::Esc, ..
            } => Key(CoreKey::Esc),
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => Key(CoreKey::Left),
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => Key(CoreKey::Right),
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => Key(CoreKey::Down),
            KeyEvent {
                code: KeyCode::Up, ..
            } => Key(CoreKey::Up),
            KeyEvent {
                code: KeyCode::Insert,
                ..
            } => Key(CoreKey::Insert),
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => Key(CoreKey::Delete),
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => Key(CoreKey::Home),
            KeyEvent {
                code: KeyCode::End, ..
            } => Key(CoreKey::End),
            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => Key(CoreKey::PageUp),
            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => Key(CoreKey::PageDown),
            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char(ch),
                ..
            } => Key(CoreKey::Ctrl(ch)),
            KeyEvent {
                code: KeyCode::Char(ch),
                ..
            } => Key(CoreKey::Char(ch)),
            _ => Key(CoreKey::Unknown),
        }
    }
}
