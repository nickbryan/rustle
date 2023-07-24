use crate::mode::Mode;
use std::fmt;

/// `Message` is an enum that captures all messages that the `Editor` and its `Component`s
/// understand.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    AbortCommandLineInput,
    EndCommandLineInput,
    ParseCommandLineInput(String),

    EnterMode(Mode),

    InsertChar(char),
    InsertLineBreak,
    DeleteCharForward,
    DeleteCharBackward,

    MoveCursorUp(usize),
    MoveCursorDown(usize),
    MoveCursorLeft(usize),
    MoveCursorRight(usize),
    MoveCursorLineStart,
    MoveCursorLineEnd,
    MoveCursorPageUp,
    MoveCursorPageDown,

    Save,
    SaveAs(String),

    Quit,
}

/// This trait is just a wrapper for our `Command` closer so that we can implement `std::fmt::Debug` on it.
pub trait CommandClosure: FnOnce() -> Message + Send + 'static {}
impl<F> CommandClosure for F where F: FnOnce() -> Message + Send + 'static {}

/// `Command` is a type alias for a closure that returns a message. This allows for more powerful
/// `Message` computation such as timers. `Command`s will be executed asynchronously by the `Editor`.
pub type Command = Box<dyn CommandClosure<Output = Message>>;

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command")
    }
}

/// Reduces some boiler plate by not having to `Box` every closure.
pub fn wrap(msg: Message) -> Command {
    Box::new(|| msg)
}
