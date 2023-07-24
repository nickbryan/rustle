use crate::{
    render::View,
    ui::{Color, Rect},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Welcome {
    pub size: Rect,
}

impl View for Welcome {
    fn render_to(&self, frame: &mut crate::render::Frame) {
        let mut message = format!("Velm editor -- version {}", VERSION);
        let padding = self.size.width.saturating_sub(message.len()) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        message = format!("~{}{}", spaces, message);
        message.truncate(self.size.width);
        for row in 0..self.size.height - 1 {
            if row == self.size.height / 3 {
                frame.write_line(row, &message, Color::default(), Color::default());
                continue;
            }

            frame.write_line(row, "~", Color::Gray, Color::default());
        }
    }
}
