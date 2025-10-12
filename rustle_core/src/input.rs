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

#[derive(Deserialize, Eq, Hash, PartialEq, Default, Clone)]
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

pub type ModeKeyBindingMap = HashMap<Mode, KeyBindingMap>;

pub(crate) struct Processor {
    key_buffer: Vec<Key>,
    bindings: ModeKeyBindingMap,
}

impl Processor {
    pub(crate) fn new(bindings: ModeKeyBindingMap) -> Self {
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

        // TODO: does this need to be cloned?
        let mut current_map_opt = self.bindings.get(mode).cloned();

        // TODO: is this efficient?
        // TODO: can we simplify this block with self.key_buffer.iter().map or something?
        for i in 0..self.key_buffer.len() {
            let key = &self.key_buffer[i];

            if let Some(current_map) = current_map_opt {
                let key_str = key.to_string();

                match current_map.get(&key_str) {
                    Some(KeyBinding::Action(action)) => {
                        self.clear();
                        return parse_action(action);
                    }
                    Some(KeyBinding::Chord(next_map)) => {
                        current_map_opt = Some(next_map.clone());
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
