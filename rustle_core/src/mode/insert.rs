use crate::communication::Message;
use crate::mode::{Mode, Normal};
use crate::Key;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Insert;

impl Insert {
    pub fn handle(key: Key) -> Option<Message> {
        match key {
            Key::Up => Some(Message::MoveCursorUp(1)),
            Key::Down => Some(Message::MoveCursorDown(1)),
            Key::Left => Some(Message::MoveCursorLeft(1)),
            Key::Right => Some(Message::MoveCursorRight(1)),
            Key::Home => Some(Message::MoveCursorLineStart),
            Key::End => Some(Message::MoveCursorLineEnd),
            Key::PageUp => Some(Message::MoveCursorPageUp),
            Key::PageDown => Some(Message::MoveCursorPageDown),
            Key::Delete => Some(Message::DeleteCharForward),
            Key::Backspace => Some(Message::DeleteCharBackward),
            Key::Enter => Some(Message::InsertLineBreak),
            Key::Char(ch) => Some(Message::InsertChar(ch)),
            Key::Esc => Some(Message::EnterMode(Mode::Normal(Normal::default()))),
            _ => None,
        }
    }
}
