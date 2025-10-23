use std::{
    collections::HashMap,
    fmt::{Display, Error as FmtError, Formatter},
    io::Error as IoError,
    pin::Pin,
};

use serde::Deserialize;

use crate::editor::{Action, Movement};

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

pub(crate) enum Resolution {
    Match(Action),
    NoMatch,
    Pending,
}

pub(crate) struct Resolver {
    bindings: ModalKeyBindingMap,
    buffer: Vec<Key>,
    multiplier: u32,
}

impl Resolver {
    pub(crate) fn new(bindings: ModalKeyBindingMap) -> Self {
        Self {
            bindings,
            buffer: Vec::new(),
            multiplier: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.buffer.clear();
        self.multiplier = 0;
    }

    pub(crate) fn resolve(&mut self, key: Key, mode: Mode) -> Resolution {
        if let Some(multiplier) = parse_multiplier(self.multiplier, key) {
            self.multiplier = multiplier;
            return Resolution::Pending;
        }

        self.buffer.push(key);

        let mut current_bindings = self.bindings.get(&mode);

        for key in &self.buffer {
            let Some(map) = current_bindings else {
                return Resolution::NoMatch;
            };

            match map.get(&key.to_string()) {
                Some(KeyBinding::Action(action)) => {
                    let multiplier = if self.multiplier == 0 {
                        1
                    } else {
                        self.multiplier
                    };
                    let result = parse_action(action, multiplier);
                    self.reset();
                    return Resolution::Match(result.unwrap());
                }
                Some(KeyBinding::Chord(next_map)) => {
                    current_bindings = Some(next_map);
                }
                None => {
                    return Resolution::NoMatch;
                }
            }
        }

        Resolution::Pending
    }

    pub(crate) fn drain_buffer(&mut self) -> String {
        self.buffer
            .drain(..)
            .map(|key: Key| key.to_string())
            .collect()
    }
}

fn parse_multiplier(current_value: u32, key: Key) -> Option<u32> {
    if let Key::Char(char) = key
        && char.is_ascii_digit()
        // 0 is a valid multiplier, but not the first
        // character in a multiplier, this can be
        // reserved for an action.
        && (char != '0' || current_value != 0)
    {
        let digit = char.to_digit(10).unwrap(); // unwrap() is safe due to is_ascii_digit().
        Some(current_value.saturating_mul(10).saturating_add(digit))
    } else {
        None
    }
}

fn parse_action(action: &str, multiplier: u32) -> Option<Action> {
    match action {
        "quit" => Some(Action::Quit),
        "enter_insert_mode" => Some(Action::EnterMode(Mode::Insert)),
        "enter_normal_mode" => Some(Action::EnterMode(Mode::Normal)),
        "move_cursor_next" => Some(Action::MoveCursor(Movement::Next(multiplier))),
        "move_cursor_prev" => Some(Action::MoveCursor(Movement::Prev(multiplier))),
        "move_line_next" => Some(Action::MoveCursor(Movement::LineNext(multiplier))),
        "move_line_prev" => Some(Action::MoveCursor(Movement::LinePrev(multiplier))),
        _ => None,
    }
}
