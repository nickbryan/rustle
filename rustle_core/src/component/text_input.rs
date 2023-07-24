use crate::communication::{self, Command, Message};
use crate::component::Component;
use crate::mode::{Mode, Normal};
use crate::render::{Frame, View};
use crate::ui::{Color, Position};
use crate::Row;
use anyhow::Result;

pub struct TextInput {
    cursor_position: usize,
    focused: bool,
    place_holder: String,
    position: Position,
    prompt: String,
    value: Row,
}

impl TextInput {
    pub fn new(prompt: &str, place_holder: &str, position: Position) -> Self {
        Self {
            cursor_position: 0,
            focused: false,
            place_holder: String::from(place_holder),
            position,
            prompt: String::from(prompt),
            value: Row::default(),
        }
    }

    /// When the `TextInput` is focused it will update the cursor position of the `Frame`
    /// to wherever the cursor should be in the `TextInput`.
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// When the `TextInput` is unfocused it will not update the cursor position of the `Frame`.
    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    fn reset(&mut self) {
        self.value = Row::default();
        self.cursor_position = 0;
    }
}

impl Component for TextInput {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        Ok(match msg {
            Message::InsertChar(ch) => {
                self.value.insert(self.cursor_position, ch);
                self.cursor_position = self.cursor_position.saturating_add(1);

                None
            }
            Message::EndCommandLineInput => {
                let cmd = Some(communication::wrap(Message::ParseCommandLineInput(
                    self.value.contents(),
                )));

                self.reset();

                cmd
            }
            Message::AbortCommandLineInput => {
                self.reset();

                Some(communication::wrap(Message::EnterMode(Mode::Normal(
                    Normal::default(),
                ))))
            }
            Message::MoveCursorLeft(n) => {
                if self.cursor_position > 1 {
                    self.cursor_position = self.cursor_position.saturating_sub(n);
                }

                None
            }
            Message::MoveCursorRight(n) => {
                if self.cursor_position != self.value.len() {
                    self.cursor_position = self.cursor_position.saturating_add(n);
                }

                None
            }
            Message::MoveCursorLineStart => {
                self.cursor_position = 1;

                None
            }
            Message::MoveCursorLineEnd => {
                self.cursor_position = self.value.len();

                None
            }
            Message::DeleteCharForward => {
                self.value.delete(self.cursor_position);

                // TODO: revmove duplication here.
                if self.value.len() <= 1 {
                    self.reset();

                    return Ok(Some(communication::wrap(Message::EnterMode(Mode::Normal(
                        Normal::default(),
                    )))));
                }

                None
            }
            Message::DeleteCharBackward => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
                self.value.delete(self.cursor_position);

                if self.value.len() <= 1 {
                    self.reset();

                    return Ok(Some(communication::wrap(Message::EnterMode(Mode::Normal(
                        Normal::default(),
                    )))));
                }

                None
            }
            _ => None,
        })
    }
}

impl View for TextInput {
    fn render_to(&self, frame: &mut Frame) {
        if self.value.is_empty() && !self.place_holder.is_empty() && !self.focused {
            frame.write_line(
                self.position.row,
                &self.place_holder,
                Color::default(),
                Color::default(),
            );

            return;
        }

        let value = format!("{}{}", self.prompt, &self.value.contents());

        frame.write_line(
            self.position.row,
            &value,
            Color::default(),
            Color::default(),
        );

        if self.focused {
            frame.set_cursor_position(Position::new(
                self.cursor_position + self.prompt.len(),
                self.position.row,
            ));
        }
    }
}
