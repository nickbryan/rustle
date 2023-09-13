use crate::{
    editor::Command,
    editor::Component,
    mode::{Mode, Normal},
    render::{Frame, View},
    ui::{Color, Position},
};
use anyhow::Result;

pub struct TextInput {
    cursor_position: usize,
    focused: bool,
    place_holder: String,
    position: Position,
    prompt: String,
    value: String,
}

impl TextInput {
    pub fn new(prompt: &str, place_holder: &str, position: Position) -> Self {
        Self {
            cursor_position: 0,
            focused: false,
            place_holder: String::from(place_holder),
            position,
            prompt: String::from(prompt),
            value: String::new(),
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
        self.value = String::new();
        self.cursor_position = 0;
    }
}

impl Component for TextInput {
    fn update(&mut self, cmd: Command) -> Result<Option<Command>> {
        Ok(match cmd {
            Command::InsertChar(ch) => {
                self.value.insert(self.cursor_position, ch);
                self.cursor_position = self.cursor_position.saturating_add(1);

                None
            }
            Command::EndCommandLineInput => {
                let cmd = Some(Command::ParseCommandLineInput(self.value.clone()));

                self.reset();

                cmd
            }
            Command::AbortCommandLineInput => {
                self.reset();

                Some(Command::EnterMode(Mode::Normal(Normal::default())))
            }
            Command::MoveCursorLeft(n) => {
                if self.cursor_position > 1 {
                    self.cursor_position = self.cursor_position.saturating_sub(n);
                }

                None
            }
            Command::MoveCursorRight(n) => {
                if self.cursor_position != self.value.len() {
                    self.cursor_position = self.cursor_position.saturating_add(n);
                }

                None
            }
            Command::MoveCursorLineStart => {
                self.cursor_position = 1;

                None
            }
            Command::MoveCursorLineEnd => {
                self.cursor_position = self.value.len();

                None
            }
            Command::DeleteCharForward => {
                self.value.remove(self.cursor_position);

                // TODO: revmove duplication here.
                if self.value.len() <= 1 {
                    self.reset();

                    return Ok(Some(Command::EnterMode(Mode::Normal(Normal::default()))));
                }

                None
            }
            Command::DeleteCharBackward => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
                self.value.remove(self.cursor_position);

                if self.value.len() <= 1 {
                    self.reset();

                    return Ok(Some(Command::EnterMode(Mode::Normal(Normal::default()))));
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
            frame.write(
                self.position,
                &self.place_holder,
                Color::default(),
                Color::default(),
            );

            return;
        }

        let value = format!("{}{}", self.prompt, &self.value.clone());

        frame.write(self.position, &value, Color::default(), Color::default());

        if self.focused {
            frame.set_cursor_position(Position::new(
                u16::try_from(self.cursor_position + self.prompt.len()).unwrap(),
                self.position.row,
            ));
        }
    }
}
