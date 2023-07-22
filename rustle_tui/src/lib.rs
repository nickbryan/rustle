use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustle_core::{Event, EventStream, Key as CoreKey};

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
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Enter,
                ..
            } => Key(CoreKey::Enter),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Tab,
                ..
            } => Key(CoreKey::Tab),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Backspace,
                ..
            } => Key(CoreKey::Backspace),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Esc,
                ..
            } => Key(CoreKey::Esc),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Left,
                ..
            } => Key(CoreKey::Left),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Right,
                ..
            } => Key(CoreKey::Right),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Down,
                ..
            } => Key(CoreKey::Down),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Up,
                ..
            } => Key(CoreKey::Up),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Insert,
                ..
            } => Key(CoreKey::Insert),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Delete,
                ..
            } => Key(CoreKey::Delete),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Home,
                ..
            } => Key(CoreKey::Home),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::End,
                ..
            } => Key(CoreKey::End),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::PageUp,
                ..
            } => Key(CoreKey::PageUp),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::PageDown,
                ..
            } => Key(CoreKey::PageDown),
            KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(ch),
                ..
            } => Key(CoreKey::Char(ch)),
            KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char(ch),
                ..
            } => Key(CoreKey::Ctrl(ch)),
            _ => Key(CoreKey::Unknown),
        }
    }
}
