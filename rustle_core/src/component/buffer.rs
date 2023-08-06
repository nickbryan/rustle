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
    offset: Position,
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
        } else if col >= self.offset.col.saturating_add(height) {
            (
                self.offset.row,
                col.saturating_sub(height).saturating_add(1),
            )
        } else {
            (self.offset.col, self.offset.row)
        };

        let offset = if col < self.offset.col {
            (col, offset.1)
        } else if col >= self.offset.col.saturating_add(width) {
            (col.saturating_add(width).saturating_add(1), offset.1)
        } else {
            (self.offset.col, offset.1)
        };

        self.offset = Position::from(offset);
    }

    fn move_cursor(&mut self, msg: &Message) {
        let terminal_height = self.viewport.height - 2;
        let Position { col, row } = self.cursor_position;
        let height = self.document.len();
        let width = self.document.row(row).map_or(0, |r| r.len_chars());

        let (col, row) = match msg {
            Message::MoveCursorUp(n) => (col, row.saturating_sub(*n)),
            Message::MoveCursorDown(n) => {
                if row < height {
                    (col, row.saturating_add(*n))
                } else {
                    (col, row)
                }
            }
            Message::MoveCursorLeft(n) => {
                if col > 0 {
                    (col - n, row)
                } else if row > 0 {
                    self.document
                        .row(row - 1)
                        .map_or((0, row - 1), |r| (r.len_chars(), row - 1))
                } else {
                    (col, row)
                }
            }
            Message::MoveCursorRight(n) => {
                if col < width {
                    (col + n, row)
                } else if row < height {
                    (0, row + n)
                } else {
                    (col, row)
                }
            }
            Message::MoveCursorPageUp => {
                if row > terminal_height {
                    (col, row - terminal_height)
                } else {
                    (col, 0)
                }
            }
            Message::MoveCursorPageDown => {
                if row.saturating_add(terminal_height) < height {
                    (col, row + terminal_height)
                } else {
                    (col, height)
                }
            }
            Message::MoveCursorLineStart => (0, row),
            Message::MoveCursorLineEnd => (width, row),
            _ => (col, row),
        };

        let new_width = self.document.row(row).map_or(0, |r| r.len_chars());

        self.cursor_position = Position {
            col: if col > new_width { new_width } else { col },
            row,
        };
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
                self.document.insert(&self.cursor_position, ch);

                self.move_cursor(&Message::MoveCursorRight(1));
            }
            Message::InsertLineBreak => {
                self.document.insert_newline(&self.cursor_position);
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
                    row_in_view + self.offset.row,
                    self.margin_width() - 1
                )
                .as_str(),
                Color::Rgb(113, 105, 95),
                Color::default(),
            );

            if let Some(row) = self.document.row(row_in_view + self.offset.row) {
                // let start = self.offset.col; // TODO: fix this and look at string conversion (should I pass rope slices?)
                // let end = self.offset.col + self.viewport.width;
                let row = row.to_string();
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
