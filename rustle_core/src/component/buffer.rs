use crate::document::{Cursor, Direction};
use crate::{
    communication::{Command, Message},
    document::Document,
    editor::Component,
    render::View,
    ui::{Color, Position, Rect},
};
use anyhow::Result;

#[derive(Debug, Default)]
struct Offset {
    pub col: usize,
    pub row: usize,
}

pub(crate) struct Buffer {
    document: Document,
    cursor_offset: Offset,
    viewport: Rect,
}

impl Buffer {
    pub(crate) fn new(viewport: Rect, document: Document) -> Self {
        Self {
            document,
            cursor_offset: Offset::default(),
            viewport,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.document.len()
    }

    pub(crate) fn cursor_position(&self) -> Position {
        Position::new(
            self.margin_width().saturating_add(
                self.document
                    .cursor()
                    .col
                    .saturating_sub(self.cursor_offset.col)
                    .try_into()
                    .unwrap(),
            ),
            self.document
                .cursor()
                .row
                .saturating_sub(self.cursor_offset.row)
                .try_into()
                .unwrap(),
        )
    }

    pub(crate) fn scroll(&mut self) {
        let Cursor { col, row } = self.document.cursor();
        let width = self.viewport.width - self.margin_width();
        let height = self.viewport.height;

        let offset_row = if row < self.cursor_offset.row {
            row
        } else if row < self.cursor_offset.row.saturating_add(5) {
            self.cursor_offset.row.saturating_sub(1)
        } else if row
            >= self
                .cursor_offset
                .row
                .saturating_add(height.saturating_sub(6).into())
        {
            row.saturating_sub(height.saturating_sub(5).into())
                .saturating_add(1)
        } else {
            self.cursor_offset.row
        };

        let offset_col = if col < self.cursor_offset.col {
            col.saturating_sub(5)
        } else if col < self.cursor_offset.col.saturating_add(5) {
            self.cursor_offset.col.saturating_sub(1)
        } else if col
            >= self
                .cursor_offset
                .col
                .saturating_add(width.saturating_sub(5).into())
        {
            col.saturating_sub(width.saturating_sub(5).into())
                .saturating_add(1)
        } else {
            self.cursor_offset.col
        };

        self.cursor_offset = Offset {
            col: offset_col,
            row: offset_row,
        };
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
                    .move_cursor_vertically(Direction::Backward(self.viewport.height.into()));
            }
            Message::MoveCursorPageDown => {
                self.document
                    .move_cursor_vertically(Direction::Forward(self.viewport.height.into()));
            }
            Message::MoveCursorLineStart => self.document.move_cursor_to_line_start(),
            Message::MoveCursorLineEnd => self.document.move_cursor_to_line_end(),
            _ => {}
        };

        self.scroll();
    }

    fn margin_width(&self) -> u16 {
        u16::try_from(
            self.document
                .len()
                .to_string()
                .len()
                .saturating_add(1)
                .max(3),
        )
        .unwrap()
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
                Position::new(0, self.viewport.top().saturating_add(row_in_view)),
                format!(
                    "{:1$} ",
                    (usize::from(row_in_view) + self.cursor_offset.row).saturating_add(1),
                    usize::from(self.margin_width().saturating_sub(1))
                )
                .as_str(),
                Color::Rgb(113, 105, 95),
                Color::default(),
            );

            if let Some(row) = self
                .document
                .line(usize::from(row_in_view) + self.cursor_offset.row)
            {
                let start = self.cursor_offset.col;
                if start <= row.len_chars() {
                    let end = start + usize::from(self.viewport.width);
                    let row = row.slice(start..end.min(row.len_chars())).to_string();
                    frame.write(
                        Position::new(
                            self.margin_width(),
                            self.viewport.top().saturating_add(row_in_view),
                        ),
                        &row,
                        Color::Rgb(236, 226, 195),
                        Color::default(),
                    );
                }
            } else {
                frame.write(
                    Position::new(0, self.viewport.top().saturating_add(row_in_view)),
                    "~",
                    Color::Rgb(74, 68, 65),
                    Color::default(),
                );
            }
        }
    }
}
