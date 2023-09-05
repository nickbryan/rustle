use crate::render::{Frame, View};
use crate::ui::{Color, Position, Rect};

pub struct StatusBar {
    pub area: Rect,
    pub mode: String,
    pub line_count: usize,
    pub cursor_position: Position,
    pub file_name: String,
}

impl View for StatusBar {
    fn render_to(&self, frame: &mut Frame) {
        let mut status = format!("Mode: [{}]    File: {}", self.mode, self.file_name);
        let line_indicator = format!(
            "L: {}/{} C: {}",
            self.cursor_position
                .row
                .saturating_sub(self.area.top())
                .saturating_add(1),
            self.line_count,
            self.cursor_position
                .col
                .saturating_sub(self.area.left())
                .saturating_add(1)
        );

        let len = status.len() + line_indicator.len();

        if usize::from(self.area.width) > len {
            status.push_str(&" ".repeat(usize::from(self.area.width) - len));
        }

        status = format!("{status}{line_indicator}");
        status.truncate(usize::from(self.area.width));

        frame.write(
            Position::new(self.area.left(), self.area.top()),
            &status,
            Color::Rgb(128, 119, 106),
            Color::Rgb(59, 56, 54),
        );
    }
}
