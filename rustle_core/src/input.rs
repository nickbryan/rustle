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

// TODO: do we need nom parser? It looks like we could be fine without. Maybe for command mode parsing?
// TODO: how do we show the current chord / pending unput? Would this be an action and a render component for it?
// TODO: is there a better name than Processor for this? Are we encapsulating the domain concept correctly?
pub(crate) struct KeyMappingEngine {
    bindings: ModalKeyBindingMap,
    key_buffer: Vec<Key>,
    multiplier: Option<u32>,
}

impl KeyMappingEngine {
    pub(crate) fn new(bindings: ModalKeyBindingMap) -> Self {
        Self {
            bindings,
            key_buffer: Vec::new(),
            multiplier: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        // TODO: fix the multiplier getting reset on idle timeout in editor.rs Editor.consume method. Do we need to tell it to wait or something?
        // TODO: do we even need idle-timeout at all? Can we just have esc clear the current chord? Or maybe if jj was mapped and jk was pressed then char j and k would be returned as actions, we would need to be able to return an action that nests multiple actions for this.
        // TODO: I guess an idle timeout is mainly for situations like in insert mode when we want to exit with jj but still want to be able to input j as a character. How would this work in the toml config? I don't think we can have an entry for j = "action" and another for j = { j = "action" } in toml.
        // TODO: how do we end a word with j and also exit insert mode? so boj followed by j would exit insert mode and then j would not be inserted.
        self.multiplier = None;
        self.key_buffer.clear();
    }

    pub(crate) fn process(&mut self, key: Key, mode: Mode) -> Option<Action> {
        if let Key::Char(char) = key
            && char.is_ascii_digit()
            // 0 is a valid multiplier, but not the first
            // character in a multiplier, this can be
            // reserved for an action.
            && (char != '0' || self.multiplier.is_some())
        {
            let multiplier_val = self.multiplier.get_or_insert(0);

            *multiplier_val = multiplier_val
                .saturating_mul(10)
                .saturating_add(char.to_digit(10).unwrap()); // unwrap() is safe due to is_ascii_digit().

            return None;
        }

        self.key_buffer.push(key);

        let bindings = &self.bindings;
        let mut current_map = bindings.get(&mode);

        for key in &self.key_buffer {
            if let Some(map) = current_map {
                let key_str = key.to_string();

                match map.get(&key_str) {
                    Some(KeyBinding::Action(action)) => {
                        let result = parse_action(action, self.multiplier.unwrap_or(1));
                        self.reset();
                        return result;
                    }
                    Some(KeyBinding::Chord(next_map)) => {
                        current_map = Some(next_map);
                    }
                    None => {
                        self.reset();
                        return None;
                    }
                }
            } else {
                self.reset();
                return None;
            }
        }

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
