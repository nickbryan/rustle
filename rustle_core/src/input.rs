use std::io::Error as IoError;
use std::pin::Pin;
use tokio_stream::Stream;

/// `Key` presses accepted by the editor.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Key {
    Backspace,
    Delete,
    Char(char),
    Ctrl(char),
    Enter,
    Esc,
    Home,
    End,
    Insert,
    PageUp,
    PageDown,
    Tab,
    Up,
    Down,
    Left,
    Right,
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
pub type EventStream = Pin<Box<dyn Stream<Item = Event> + Send>>;
