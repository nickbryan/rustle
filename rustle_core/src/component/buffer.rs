use crate::document::Direction;
use crate::{
    communication::{Command, Message},
    document::Document,
    editor::Component,
    render::View,
    ui::{Color, Position, Rect},
};
use anyhow::Result;

pub struct Buffer {
    cursor_position: Position,
    document: Document,
    pub offset: Position,
    viewport: Rect,
}

impl Buffer {
    pub fn len(&self) -> usize {
        self.document.len()
    }

    pub fn new(viewport: Rect, document: Document) -> Self {
        Self {
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            viewport,
        }
    }

    pub fn cursor_position(&self) -> Position {
        Position::new(
            self.margin_width() + self.cursor_position.col.saturating_sub(self.offset.col),
            self.cursor_position.row.saturating_sub(self.offset.row),
        )
    }

    pub fn scroll(&mut self) {
        let Position { col, row } = self.cursor_position;
        let width = self.viewport.width;
        let height = self.viewport.height - 2;

        let offset = if row < self.offset.row {
            (self.offset.col, row)
        } else if row < self.offset.row + 5 {
            (self.offset.col, self.offset.row.saturating_sub(1))
        } else if row >= self.offset.row.saturating_add(height - 5) {
            (
                self.offset.col,
                row.saturating_sub(height - 5).saturating_add(1),
            )
        } else {
            (self.offset.col, self.offset.row)
        };

        let offset = if row < self.offset.row {
            (col, offset.1)
        } else if col < self.offset.col + 5 {
            (self.offset.col.saturating_sub(1), offset.1)
        } else if col >= self.offset.col.saturating_add(width - 5) {
            (col.saturating_sub(height - 5).saturating_add(1), offset.1)
        } else {
            (self.offset.col, self.offset.row)
        };

        self.offset = Position::from(offset);
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
                self.document.move_cursor_vertically(Direction::Backward(1));
            }
            Message::MoveCursorPageDown => {
                self.document.move_cursor_vertically(Direction::Forward(1));
            }
            Message::MoveCursorLineStart | Message::MoveCursorLineEnd => {}
            _ => {}
        };

        self.cursor_position = self.document.cursor_coordinates();
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
            Message::InsertChar(ch) => {
                self.document.insert(ch);

                self.move_cursor(&Message::MoveCursorRight(1));
            }
            Message::InsertLineBreak => {
                self.document.insert_newline();
                self.move_cursor(&Message::MoveCursorDown(1));
                self.move_cursor(&Message::MoveCursorLineStart);
            }
            Message::DeleteCharForward => self.document.delete(&self.cursor_position),
            Message::DeleteCharBackward => {
                if self.cursor_position.col > 0 || self.cursor_position.row > 0 {
                    self.move_cursor(&Message::MoveCursorLeft(1));
                    self.document.delete(&self.cursor_position);
                }
            }
            _ => {
                self.move_cursor(&msg);
            }
        };

        self.scroll();

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
                // let start = self.offset.col; // TODO: fix this and look at string conversion (should I pass rope slices?)
                // let end = self.offset.col + self.viewport.width;
                let row = row
                    .slice(0..self.viewport.width.min(row.len_chars()))
                    .to_string();
                frame.write(
                    &Position {
                        col: self.margin_width(),
                        row: row_in_view,
                    },
                    &row,
                    Color::Rgb(236, 226, 195),
                    Color::default(),
                );
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
