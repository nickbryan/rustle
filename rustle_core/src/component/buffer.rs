use crate::document::Direction;
use crate::{
    communication::{Command, Message},
    document::Document,
    editor::Component,
    render::View,
    ui::{Color, Position, Rect},
};
use anyhow::Result;

pub(crate) struct Buffer {
    document: Document,
    pub offset: Position,
    viewport: Rect,
}

impl Buffer {
    pub(crate) fn len(&self) -> usize {
        self.document.len()
    }

    pub(crate) fn new(viewport: Rect, document: Document) -> Self {
        Self {
            document,
            offset: Position::default(),
            viewport,
        }
    }

    pub(crate) fn cursor_position(&self) -> Position {
        Position::new(
            self.margin_width()
                + self
                    .document
                    .cursor_coordinates()
                    .col
                    .saturating_sub(self.offset.col),
            self.document
                .cursor_coordinates()
                .row
                .saturating_sub(self.offset.row),
        )
    }

    pub(crate) fn scroll(&mut self) {
        let Position { col, row } = self.document.cursor_coordinates();
        let width = self.viewport.width - self.margin_width();
        let height = self.viewport.height - 2;

        let offset_row = if row < self.offset.row {
            row
        } else if row < self.offset.row + 5 {
            self.offset.row.saturating_sub(1)
        } else if row >= self.offset.row.saturating_add(height - 5) {
            row.saturating_sub(height - 5).saturating_add(1)
        } else {
            self.offset.row
        };

        let offset_col = if col < self.offset.col {
            col.saturating_sub(5)
        } else if col < self.offset.col + 5 {
            self.offset.col.saturating_sub(1)
        } else if col >= self.offset.col.saturating_add(width - 5) {
            col.saturating_sub(width - 5).saturating_add(1)
        } else {
            self.offset.col
        };

        self.offset = Position::new(offset_col, offset_row);
    }

    fn move_cursor(&mut self, msg: &Message) {
        match msg {
            Message::MoveCursorUp(n) => {
                self.document
                    .move_cursor_vertically(Direction::Backward(*n));
            }
            Message::MoveCursorDown(n) => {
                self.document.move_cursor_vertically(Direction::Forward(*n));
            }
            Message::MoveCursorLeft(n) => {
                self.document
                    .move_cursor_horizontally(Direction::Backward(*n));
            }
            Message::MoveCursorRight(n) => {
                self.document
                    .move_cursor_horizontally(Direction::Forward(*n));
            }
            Message::MoveCursorPageUp => {
                self.document
                    .move_cursor_vertically(Direction::Backward(self.viewport.height));
            }
            Message::MoveCursorPageDown => {
                self.document
                    .move_cursor_vertically(Direction::Forward(self.viewport.height));
            }
            Message::MoveCursorLineStart => self.document.move_cursor_to_line_start(),
            Message::MoveCursorLineEnd => self.document.move_cursor_to_line_end(),
            _ => {}
        };

        self.scroll();
    }

    fn margin_width(&self) -> usize {
        self.document
            .len()
            .to_string()
            .len()
            .saturating_add(1)
            .max(3)
    }
}

impl Component for Buffer {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        match msg {
            Message::InsertChar(ch) => self.document.insert(ch),
            Message::InsertLineBreak => self.document.insert_newline(),
            Message::DeleteCharForward => self.document.delete(Direction::Forward(1)),
            Message::DeleteCharBackward => self.document.delete(Direction::Backward(1)),
            _ => {
                self.move_cursor(&msg);
            }
        };

        Ok(None)
    }
}

impl View for Buffer {
    fn render_to(&self, frame: &mut crate::render::Frame) {
        for row_in_view in 0..self.viewport.height {
            frame.write(
                &Position {
                    col: 0,
                    row: row_in_view,
                },
                format!(
                    "{:1$} ",
                    (row_in_view + self.offset.row).saturating_add(1),
                    self.margin_width().saturating_sub(1)
                )
                .as_str(),
                Color::Rgb(113, 105, 95),
                Color::default(),
            );

            if let Some(row) = self.document.line(row_in_view + self.offset.row) {
                let start = self.offset.col;
                if start <= row.len_chars() {
                    let end = start + self.viewport.width;
                    let row = row.slice(start..end.min(row.len_chars())).to_string();
                    frame.write(
                        &Position {
                            col: self.margin_width(),
                            row: row_in_view,
                        },
                        &row,
                        Color::Rgb(236, 226, 195),
                        Color::default(),
                    );
                }
            } else {
                frame.write(
                    &Position {
                        col: 0,
                        row: row_in_view,
                    },
                    "~",
                    Color::Rgb(74, 68, 65),
                    Color::default(),
                );
            }
        }
    }
}
