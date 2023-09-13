use crate::{
    editor::Command,
    mode::{Mode, Normal},
    Key,
};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Insert;

impl Insert {
    pub fn handle(key: Key) -> Option<Command> {
        match key {
            Key::Up => Some(Command::MoveCursorUp(1)),
            Key::Down => Some(Command::MoveCursorDown(1)),
            Key::Left => Some(Command::MoveCursorLeft(1)),
            Key::Right => Some(Command::MoveCursorRight(1)),
            Key::Home => Some(Command::MoveCursorLineStart),
            Key::End => Some(Command::MoveCursorLineEnd),
            Key::PageUp => Some(Command::MoveCursorPageUp),
            Key::PageDown => Some(Command::MoveCursorPageDown),
            Key::Delete => Some(Command::DeleteCharForward),
            Key::Backspace => Some(Command::DeleteCharBackward),
            Key::Enter => Some(Command::InsertLineBreak),
            Key::Char(ch) => Some(Command::InsertChar(ch)),
            Key::Esc => Some(Command::EnterMode(Mode::Normal(Normal::default()))),
            _ => None,
        }
    }
}
