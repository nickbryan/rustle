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
        let message = format!("🍂  Rustle editor -- version {VERSION}");
        let padding = self.size.width.saturating_sub(message.len()) / 2;
        frame.write(
            &Position {
                col: padding,
                row: self.size.height / 3,
            },
            &message,
            Color::default(),
            Color::default(),
        );
    }
}
