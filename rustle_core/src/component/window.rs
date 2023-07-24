use crate::communication::{Command, Message};
use crate::component::{Buffer, Component, StatusBar, TextInput, Welcome};
use crate::document::Document;
use crate::mode::Mode;
use crate::render::{Frame, View};
use crate::ui::{Position, Rect};
use anyhow::Result;

/// `Window` is the default root component for the `Editor`.
pub struct Window {
    active_buffer_idx: usize,
    buffers: Vec<Buffer>,
    command_prompt: TextInput,
    mode: Mode,
    size: Rect,
}

impl Window {
    pub fn new(size: Rect, mode: Mode) -> Self {
        let mut command_prompt = TextInput::new(
            ":",
            " Press : to enter a command...",
            Position::new(0, size.bottom()),
        );

        if let Mode::Execute(_) = mode {
            command_prompt.focus();
        }

        Self {
            active_buffer_idx: 0,
            buffers: Vec::default(),
            command_prompt,
            mode,
            size,
        }
    }

    fn buffer_space(&self) -> Rect {
        Rect::positioned(
            self.size.width,
            self.size.height,
            self.size.left(),
            self.size.bottom() - 2,
        )
    }
}

impl Component for Window {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        if let Message::EnterMode(mode) = msg.clone() {
            if let Mode::Insert(_) = mode {
                if self.buffers.is_empty() {
                    self.buffers
                        .push(Buffer::new(self.buffer_space(), Document::default()));
                }
            }

            if let Mode::Execute(_) = mode {
                self.command_prompt.focus();
            } else {
                self.command_prompt.unfocus();
            }

            self.mode = mode;
        }

        if let Mode::Execute(_) = self.mode {
            return self.command_prompt.update(msg);
        }

        self.buffers[self.active_buffer_idx].update(msg)
    }
}

impl View for Window {
    fn render_to(&self, frame: &mut Frame) {
        if self.buffers.is_empty() {
            Welcome {
                size: self.buffer_space(),
            }
            .render_to(frame);
        } else {
            self.buffers[self.active_buffer_idx].render_to(frame);
        }

        if let Mode::Normal(_) | Mode::Insert(_) = self.mode {
            frame.set_cursor_position(if self.buffers.is_empty() {
                Position::default()
            } else {
                self.buffers[self.active_buffer_idx].cursor_position()
            });
        }

        StatusBar {
            area: Rect::positioned(self.size.width, 1, self.size.left(), self.size.bottom() - 1),
            mode: self.mode.to_string(),
            line_count: 0,
            cursor_position: frame.cursor_position(),
            file_name: "".to_string(),
        }
        .render_to(frame);

        self.command_prompt.render_to(frame);
    }
}
