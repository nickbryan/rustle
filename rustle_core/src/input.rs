use std::io::Error as IoError;
use std::pin::Pin;

/// `Key` presses accepted by the editor.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Key {
    Enter,
    Tab,
    Backspace,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Char(char),
    Ctrl(char),
    Unknown,
}

/// `Event`s are dispatched from the backend to allow the application to handle input.
#[derive(Debug)]
pub enum Event {
    KeyPressed(Key),
    MouseInputReceived,
    WindowResized(u16, u16),
    ReadFailed(IoError),
}

/// `EventStream` is a an asynchronous tokio stream of input Events.
pub type EventStream = Pin<Box<dyn tokio_stream::Stream<Item = Event> + Send>>;
