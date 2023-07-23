use crate::communication::{Command, Message};
use crate::editor::Component;
use crate::render::{Frame, View};
use crate::ui::{Color, Rect};
use anyhow::Result;

/// `Window` is the default root component for the `Editor`.
pub struct Window {
    size: Rect,
}

impl Window {
    pub fn new(size: Rect) -> Self {
        Self { size }
    }
}

impl Component for Window {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        Ok(None)
    }
}

impl View for Window {
    fn render_to(&self, frame: &mut Frame) {
        frame.write_line(0, "Testing 123...", Color::White, Color::DarkGray);
    }
}
