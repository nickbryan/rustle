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
            self.cursor_position.row,
            self.line_count,
            self.cursor_position.col + 1
        );

        let len = status.len() + line_indicator.len();

        if self.area.width > len {
            status.push_str(&" ".repeat(self.area.width - len));
        }

        status = format!("{}{}", status, line_indicator);
        status.truncate(self.area.width);

        frame.write_line(
            self.area.top(),
            &status,
            Color::Rgb(63, 63, 63),
            Color::Rgb(239, 239, 239),
        );
    }
}
