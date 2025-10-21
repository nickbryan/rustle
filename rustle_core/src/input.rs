use std::{
    collections::HashMap,
    fmt::{Display, Error as FmtError, Formatter},
    io::Error as IoError,
    pin::Pin,
};

use serde::Deserialize;

use crate::editor::Action;

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

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            Key::Unknown => write!(f, "<unknown>"),

            // Character keys
            Key::Char(' ') => write!(f, "<space>"),
            Key::Char('<') => write!(f, "<lt>"),
            Key::Char('>') => write!(f, "<gt>"),
            Key::Char(c) => write!(f, "{c}"),

            // Modified keys
            Key::Ctrl(c) => write!(f, "<C-{c}>"),

            // Special keys
            Key::Enter => write!(f, "<cr>"),
            Key::Esc => write!(f, "<esc>"),
            Key::Tab => write!(f, "<tab>"),
            Key::Backspace => write!(f, "<bs>"),

            // Navigation keys
            Key::Up => write!(f, "<up>"),
            Key::Down => write!(f, "<down>"),
            Key::Left => write!(f, "<left>"),
            Key::Right => write!(f, "<right>"),
            Key::Home => write!(f, "<home>"),
            Key::End => write!(f, "<end>"),
            Key::PageUp => write!(f, "<pageup>"),
            Key::PageDown => write!(f, "<pagedown>"),
            Key::Insert => write!(f, "<insert>"),
            Key::Delete => write!(f, "<delete>"),
        }
    }
}

/// `Event` is dispatched from the backend to allow the application to handle input.
#[derive(Debug)]
pub enum Event {
    /// A key was pressed.
    KeyPressed(Key),
    /// A mouse input event was received.
    MouseInputReceived,
    /// The window was resized.
    WindowResized(u16, u16),
    /// An error occurred while reading input.
    ReadFailed(IoError),
}

/// `EventStream` is an asynchronous tokio stream of input Events.
pub type EventStream = Pin<Box<dyn tokio_stream::Stream<Item = Event> + Send>>;

#[derive(Deserialize, Eq, Hash, PartialEq, Default, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            Self::Insert => write!(f, "INSERT"),
            Self::Normal => write!(f, "NORMAL"),
        }
    }
}

pub type KeyBindingMap = HashMap<String, KeyBinding>;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum KeyBinding {
    Action(String),
    Chord(KeyBindingMap),
}

pub type ModalKeyBindingMap = HashMap<Mode, KeyBindingMap>;

pub(crate) struct Processor {
    key_buffer: Vec<Key>,
    bindings: ModalKeyBindingMap,
}

impl Processor {
    pub(crate) fn new(bindings: ModalKeyBindingMap) -> Self {
        Self {
            key_buffer: Vec::new(),
            bindings,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.key_buffer.clear();
    }

    pub(crate) fn process(&mut self, key: Key, mode: &Mode) -> Option<Action> {
        self.key_buffer.push(key);

        let bindings = &self.bindings;
        let mut current_map = bindings.get(mode);

        for key in &self.key_buffer {
            if let Some(map) = current_map {
                let key_str = key.to_string();

                match map.get(&key_str) {
                    Some(KeyBinding::Action(action)) => {
                        let result = parse_action(action);
                        self.clear();
                        return result;
                    }
                    Some(KeyBinding::Chord(next_map)) => {
                        current_map = Some(next_map);
                    }
                    None => {
                        self.clear();
                        return None;
                    }
                }
            } else {
                self.clear();
                return None;
            }
        }

        None
    }
}

fn parse_action(action: &str) -> Option<Action> {
    match action {
        "quit" => Some(Action::Quit),
        "enter_insert_mode" => Some(Action::EnterMode(Mode::Insert)),
        "enter_normal_mode" => Some(Action::EnterMode(Mode::Normal)),
        _ => None,
    }
}
