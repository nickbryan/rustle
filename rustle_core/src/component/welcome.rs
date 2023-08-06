use crate::ui::Position;
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
        let mut message = format!("Rustle editor -- version {VERSION}");
        let padding = self.size.width.saturating_sub(message.len()) / 2;
        let spaces = " ".repeat(padding);
        message = format!("{spaces}{message}");
        message.truncate(self.size.width);
        for row in 0..self.size.height - 1 {
            if row == self.size.height / 3 {
                frame.write(
                    &Position { col: 0, row },
                    &message,
                    Color::default(),
                    Color::default(),
                );
                continue;
            }
        }
    }
}
