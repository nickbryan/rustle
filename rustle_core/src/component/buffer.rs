use crate::{
    communication::{Command, Message},
    component::Component,
    document::Document,
    render::View,
    ui::{Color, Position, Rect},
};
use anyhow::Result;

pub struct Buffer {
    cursor_position: Position,
    document: Document,
    focused: bool,
    offset: Position,
    viewport: Rect,
}

impl Buffer {
    pub fn new(viewport: Rect, document: Document) -> Self {
        Self {
            cursor_position: Position::default(),
            document,
            focused: false,
            offset: Position::default(),
            viewport,
        }
    }

    pub fn document_name(&self) -> String {
        self.document
            .file_name()
            .unwrap_or(&"[No Name]".to_string())
            .clone()
    }

    pub fn cursor_position(&self) -> Position {
        Position::new(
            self.cursor_position.col.saturating_sub(self.offset.col),
            self.cursor_position.row.saturating_sub(self.offset.row),
        )
    }

    pub fn lines_in_document(&self) -> usize {
        self.document.len()
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

    fn move_cursor(&mut self, msg: Message) {
        use crate::Row;

        let terminal_height = self.viewport.height - 2;
        let Position { col, row } = self.cursor_position;
        let height = self.document.len();
        let width = self.document.row(row).map_or(0, Row::len);

        let (col, row) = match msg {
            Message::MoveCursorUp(n) => (col, row.saturating_sub(n)),
            Message::MoveCursorDown(n) => {
                if row < height {
                    (col, row.saturating_add(n))
                } else {
                    (col, row)
                }
            }
            Message::MoveCursorLeft(n) => {
                if col > 0 {
                    (col - n, row)
                } else if row > 0 {
                    self.document
                        .row(row)
                        .map_or((0, row - n), |r| (r.len(), row - n))
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

        let new_width = self.document.row(row).map_or(0, Row::len);

        self.cursor_position = Position {
            col: if col > new_width { new_width } else { col },
            row,
        };
    }
}

impl Component for Buffer {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        use anyhow::Context;

        match msg {
            Message::InsertChar(ch) => {
                self.document
                    .insert(&self.cursor_position, ch)
                    .context("unable to insert character in document")?;

                self.move_cursor(Message::MoveCursorRight(1));
            }
            Message::InsertLineBreak => {
                self.document.insert_newline(&self.cursor_position);
                self.move_cursor(Message::MoveCursorDown(1));
                self.move_cursor(Message::MoveCursorLineStart);
            }
            Message::DeleteCharForward => self.document.delete(&self.cursor_position),
            Message::DeleteCharBackward => {
                if self.cursor_position.col > 0 || self.cursor_position.row > 0 {
                    self.move_cursor(Message::MoveCursorLeft(1));
                    self.document.delete(&self.cursor_position);
                }
            }

            Message::Save => self
                .document
                .save(None)
                .context("unable to save document")?,
            Message::SaveAs(filename) => self
                .document
                .save(Some(&filename))
                .context("unable to save document")?,
            _ => {
                self.move_cursor(msg);
            }
        };

        self.scroll();

        Ok(None)
    }
}

impl View for Buffer {
    fn render_to(&self, frame: &mut crate::render::Frame) {
        if self.focused {
            frame.set_cursor_position(self.cursor_position);
        }

        for row_in_view in 0..self.viewport.height {
            if let Some(row) = self.document.row(row_in_view as usize + self.offset.row) {
                let start = self.offset.col;
                let end = self.offset.col + self.viewport.width;
                let row = row.to_string(start, end);
                frame.write_line(row_in_view, &row, Color::default(), Color::default());
            } else {
                frame.write_line(row_in_view, "~", Color::Gray, Color::default());
            }
        }
    }
}
