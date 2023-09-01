use crate::communication::{Command, Message};
use crate::editor::Component;
use crate::graphemes::RopeExt;
use crate::render::View;
use crate::ui::{Color, Position, Rect};
use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::io::Read;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Direction {
    Forward(usize),
    Backward(usize),
}

#[derive(Debug, Default)]
pub(crate) struct Selection {
    anchor: usize,
    head: usize,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct Cursor {
    pub(crate) col: usize,
    pub(crate) row: usize,
}

impl Cursor {
    fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Document {
    text: Rope,
    selection: Selection,
    desired_visual_col: usize,
    cursor_offset: Cursor,
    viewport: Rect,
}

impl Document {
    // TODO: test
    pub(crate) fn new(viewport: Rect) -> Self {
        Self {
            text: Rope::default(),
            selection: Selection::default(),
            desired_visual_col: 0,
            viewport,
            cursor_offset: Cursor::default(),
        }
    }

    // TODO: test REsult
    pub(crate) fn from(viewport: Rect, reader: impl Read) -> Result<Self> {
        Ok(Self {
            text: Rope::from_reader(reader).context("creating rope from reader")?,
            selection: Selection::default(),
            desired_visual_col: 0,
            viewport,
            cursor_offset: Cursor::default(),
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.text.len_lines()
    }

    fn cursor(&self) -> Cursor {
        Cursor::new(
            self.text.visual_column_position(self.selection.head),
            self.text.char_to_line(self.selection.head),
        )
    }

    fn move_cursor_horizontally(&mut self, direction: Direction) {
        match direction {
            Direction::Forward(chars) => {
                self.selection.head = self
                    .text
                    .nth_next_grapheme_boundary(self.selection.head, chars);
                self.selection.anchor = self
                    .text
                    .nth_next_grapheme_boundary(self.selection.anchor, chars);
            }
            Direction::Backward(chars) => {
                self.selection.head = self
                    .text
                    .nth_prev_grapheme_boundary(self.selection.head, chars);
                self.selection.anchor = self
                    .text
                    .nth_prev_grapheme_boundary(self.selection.anchor, chars);
            }
        };

        self.desired_visual_col = self.text.visual_column_position(self.selection.head);
    }

    fn move_cursor_vertically(&mut self, direction: Direction) {
        match direction {
            Direction::Forward(lines) => {
                let current_line = self.text.char_to_line(self.selection.head);
                let target_line = current_line
                    .saturating_add(lines)
                    .min(self.text.len_lines().saturating_sub(1));

                self.selection.head = self.text.line_to_char(target_line);

                if self.desired_visual_col > 0 {
                    self.selection.head = self
                        .text
                        .visual_column_position_to_char_idx(target_line, self.desired_visual_col);
                }

                if target_line.saturating_add(1) < self.text.len_lines() {
                    self.selection.head = self.selection.head.min(
                        self.text
                            .line_to_char(target_line.saturating_add(1))
                            .saturating_sub(1),
                    );
                }
                self.selection.anchor = self.selection.head;
            }
            Direction::Backward(lines) => {
                let current_line = self.text.char_to_line(self.selection.head);
                let target_line = current_line.saturating_sub(lines);

                self.selection.head = self.text.line_to_char(target_line);

                if self.desired_visual_col > 0 {
                    self.selection.head = self
                        .text
                        .visual_column_position_to_char_idx(target_line, self.desired_visual_col);
                }

                self.selection.head = self.selection.head.min(
                    self.text
                        .line_to_char(target_line.saturating_add(1))
                        .saturating_sub(1),
                );
                self.selection.anchor = self.selection.head;
            }
        };
    }

    fn move_cursor_to_line_start(&mut self) {
        self.selection.head = self
            .text
            .line_to_char(self.text.char_to_line(self.selection.head));
        self.selection.anchor = self.selection.head;

        self.desired_visual_col = self.text.visual_column_position(self.selection.head);
    }

    fn move_cursor_to_line_end(&mut self) {
        self.selection.head = self.text.prev_grapheme_boundary(
            self.text
                .line_to_char(self.text.char_to_line(self.selection.head) + 1),
        );
        self.selection.anchor = self.selection.head;
        self.desired_visual_col = self.text.visual_column_position(self.selection.head);
    }

    fn delete(&mut self, direction: Direction) {
        match direction {
            Direction::Forward(chars) => self.text.remove(
                self.selection.head
                    ..self
                        .text
                        .nth_next_grapheme_boundary(self.selection.head, chars),
            ),
            Direction::Backward(chars) => {
                let start = self.selection.head;
                self.move_cursor_horizontally(Direction::Backward(chars));
                self.text.remove(self.selection.head..start);
            }
        };
    }

    fn insert(&mut self, ch: char) {
        self.text.insert_char(self.selection.head, ch);
        self.move_cursor_horizontally(Direction::Forward(1));
    }

    fn insert_newline(&mut self) {
        self.insert('\n');
    }

    // TODO: test
    fn line(&self, line_number: usize) -> Option<RopeSlice> {
        if line_number >= self.text.len_lines() {
            // TODO: panic instead of option if bounds check fails?
            return None;
        }

        self.text.get_line(line_number).map(|slice| {
            if slice.len_chars() > 0 && slice.char(slice.len_chars() - 1) == '\n' {
                slice.slice(0..slice.len_chars() - 1)
            } else {
                slice
            }
        })
    }

    // TODO ------

    pub(crate) fn cursor_position(&self) -> Position {
        Position::new(
            self.margin_width().saturating_add(
                self.cursor()
                    .col
                    .saturating_sub(self.cursor_offset.col)
                    .try_into()
                    .unwrap(),
            ),
            self.cursor()
                .row
                .saturating_sub(self.cursor_offset.row)
                .try_into()
                .unwrap(),
        )
    }

    pub(crate) fn scroll(&mut self) {
        let Cursor { col, row } = self.cursor();
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

        self.cursor_offset = Cursor {
            col: offset_col,
            row: offset_row,
        };
    }

    fn move_cursor(&mut self, msg: &Message) {
        match msg {
            Message::MoveCursorUp(n) => {
                self.move_cursor_vertically(Direction::Backward(*n));
            }
            Message::MoveCursorDown(n) => {
                self.move_cursor_vertically(Direction::Forward(*n));
            }
            Message::MoveCursorLeft(n) => {
                self.move_cursor_horizontally(Direction::Backward(*n));
            }
            Message::MoveCursorRight(n) => {
                self.move_cursor_horizontally(Direction::Forward(*n));
            }
            Message::MoveCursorPageUp => {
                self.move_cursor_vertically(Direction::Backward(self.viewport.height.into()));
            }
            Message::MoveCursorPageDown => {
                self.move_cursor_vertically(Direction::Forward(self.viewport.height.into()));
            }
            Message::MoveCursorLineStart => self.move_cursor_to_line_start(),
            Message::MoveCursorLineEnd => self.move_cursor_to_line_end(),
            _ => {}
        };

        self.scroll();
    }

    fn margin_width(&self) -> u16 {
        u16::try_from(self.len().to_string().len().saturating_add(1).max(3)).unwrap()
    }
}

impl Component for Document {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        match msg {
            Message::InsertChar(ch) => self.insert(ch),
            Message::InsertLineBreak => self.insert_newline(),
            Message::DeleteCharForward => self.delete(Direction::Forward(1)),
            Message::DeleteCharBackward => {
                self.delete(Direction::Backward(1));
            }
            _ => {
                self.move_cursor(&msg);
            }
        };

        Ok(None)
    }
}

impl View for Document {
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

            if let Some(row) = self.line(usize::from(row_in_view) + self.cursor_offset.row) {
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

#[cfg(test)]
mod tests {
    use super::*;

    struct FileNotFoundReader;

    impl Read for FileNotFoundReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }

    #[test]
    fn document_from_reader_handles_error_with_context() {
        let error = Document::from(Rect::default(), FileNotFoundReader).unwrap_err();

        assert_eq!(
            Document::from(Rect::default(), FileNotFoundReader)
                .unwrap_err()
                .to_string(),
            "creating rope from reader"
        );
        let mut chain = error.chain();
        assert_eq!(
            chain.next().map(|x| format!("{x}")),
            Some("creating rope from reader".to_owned())
        );
        assert_eq!(
            chain.next().map(|x| format!("{x}")),
            Some("file not found".to_owned())
        );
        assert_eq!(chain.next().map(|x| format!("{x}")), None);
    }

    #[test]
    fn document_len() {
        assert_eq!(Document::default().len(), 1);
        assert_eq!(
            Document::from(Rect::default(), "1".as_bytes())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            Document::from(Rect::default(), "1\n".as_bytes())
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            Document::from(Rect::default(), "1\n2\n3\n".as_bytes())
                .unwrap()
                .len(),
            4
        );
    }

    #[test]
    fn document_move_cursor_horizontally_does_nothing_for_empty_document() {
        let mut document = Document::default();
        document.move_cursor_horizontally(Direction::Forward(0));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(10));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(0));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(10));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
    }

    #[test]
    fn document_move_cursor_horizontally_through_document() {
        let mut document = Document::from(
            Rect::default(),
            "1234\nabcd\n🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n🦀🌳🦀🌳\n".as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(0));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(2, 0));
        document.move_cursor_horizontally(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(4, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 1));
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor(), Cursor::new(0, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(2, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(4, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(6, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 3));
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor(), Cursor::new(0, 4));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 4));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 3));
        document.move_cursor_horizontally(Direction::Backward(4));
        assert_eq!(document.cursor(), Cursor::new(0, 3));
        document.move_cursor_horizontally(Direction::Backward(5));
        assert_eq!(document.cursor(), Cursor::new(0, 2));
        document.move_cursor_horizontally(Direction::Backward(10));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
    }

    #[test]
    fn document_move_cursor_vertically_does_nothing_for_empty_document() {
        let mut document = Document::default();
        document.move_cursor_vertically(Direction::Forward(0));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(10));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Backward(0));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Backward(10));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
    }

    #[test]
    fn document_move_cursor_vertically_through_first_column() {
        let mut document = Document::from(
            Rect::default(),
            "\
                        1234\n\
                        abcdefghijklmnop\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                        123\n\
                        \n\
                        \n\
                        🦀🌳🦀🌳🦀🌳🦀\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_vertically(Direction::Forward(0));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 1));
        document.move_cursor_vertically(Direction::Forward(100));
        assert_eq!(document.cursor(), Cursor::new(0, 7));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 6));
        document.move_cursor_vertically(Direction::Backward(100));
        assert_eq!(document.cursor(), Cursor::new(0, 0));
    }

    #[test]
    fn document_move_cursor_vertically_through_second_column() {
        let mut document = Document::from(
            Rect::default(),
            "\
                        1234\n\
                        abcdefghijklmnop\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                        123\n\
                        \n\
                        \n\
                        🦀🌳🦀🌳🦀🌳🦀\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 1));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 2));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 3));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 4));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 5));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 6));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 7));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(0, 7));
        document.move_cursor_vertically(Direction::Backward(2));
        assert_eq!(document.cursor(), Cursor::new(0, 5));
        document.move_cursor_vertically(Direction::Backward(2));
        assert_eq!(document.cursor(), Cursor::new(1, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 2));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 1));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 0));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 0));
    }

    #[test]
    fn document_move_cursor_vertically_aligns_to_previous_grapheme_boundary() {
        let mut document = Document::from(
            Rect::default(),
            "\
                        abcdefgh\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                        123asdas\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(7));
        assert_eq!(document.cursor(), Cursor::new(7, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(6, 1));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(6, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(7, 2));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(6, 1));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(6, 3));
        document.move_cursor_vertically(Direction::Backward(3));
        assert_eq!(document.cursor(), Cursor::new(7, 0));
    }

    #[test]
    fn document_move_cursor_vertically_aligns_to_next_grapheme_boundary() {
        // TODO: these tests, make them cleaner
        let mut document = Document::from(
            Rect::default(),
            // 🏴󠁧󠁢󠁥󠁮󠁧󠁿 has len of 7 so we align to the right grapheme boundary.
            "\
                        abcdefgh\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                        123asdas\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(8));
        assert_eq!(document.cursor(), Cursor::new(8, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 1));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(8, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 2));
    }

    #[test]
    fn document_move_cursor_vertically_handles_different_line_lengths() {
        // TODO: these tests, make them cleaner
        let mut document = Document::from(
            Rect::default(),
            // 🏴󠁧󠁢󠁥󠁮󠁧󠁿 has len of 7 so we align to the right grapheme boundary.
            "\
                        abcdefghijklmnopqrstuvwxyz\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                        123asdasasdawd\n\
                        a\n\
                        abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz\n\
                        \n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                        abcdefghijklmnopqrstuvwxyz\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(26));
        assert_eq!(document.cursor(), Cursor::new(26, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 1));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(14, 2));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 3));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(26, 4));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 5));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 6));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor(), Cursor::new(26, 7));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 6));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(0, 5));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(26, 4));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(1, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(14, 2));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(8, 1));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor(), Cursor::new(26, 0));
    }

    #[test]
    fn document_delete_forward() {
        let mut document = Document::from(
            Rect::default(),
            "\
                        abc\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor(), Cursor::new(0, 0));

        document.delete(Direction::Forward(1));
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("bc")));

        document.delete(Direction::Forward(2));
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("")));

        document.delete(Direction::Forward(1));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿")));

        document.delete(Direction::Forward(1));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿")));

        document.delete(Direction::Forward(100));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("")));
    }

    #[test]
    fn document_delete_backward() {
        let mut document = Document::from(
            Rect::default(),
            "\
                        abc\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(9));
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor(), Cursor::new(0, 2));

        document.delete(Direction::Backward(1));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(8, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿")));

        document.delete(Direction::Backward(2));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(4, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬🇧🇯🇲")));

        document.delete(Direction::Backward(100));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("")));
    }

    #[test]
    fn document_insert() {
        let mut document = Document::default();
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(0, 0));

        document.insert('h');
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(1, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("h")));
        document.insert('3');
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(2, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("h3")));
        document.insert('\n');
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(0, 1));
        assert_eq!(document.line(0), Some(RopeSlice::from("h3")));
        assert_eq!(document.line(1), Some(RopeSlice::from("")));
        document.insert('🇬');
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(1, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬")));
        document.insert('🇧');
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(2, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬🇧")));

        let mut document = Document::from(Rect::default(), "ello".as_bytes()).unwrap();
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(0, 0));

        document.insert('h');
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(1, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("hello")));
    }

    #[test]
    fn document_insert_newline() {
        let mut document = Document::from(Rect::default(), "hello".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(100));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(5, 0));

        document.insert_newline();
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(0, 1));
        assert_eq!(document.line(0), Some(RopeSlice::from("hello")));
        assert_eq!(document.line(1), Some(RopeSlice::from("")));
        document.insert_newline();
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor(), Cursor::new(0, 2));
        assert_eq!(document.line(0), Some(RopeSlice::from("hello")));
        assert_eq!(document.line(1), Some(RopeSlice::from("")));
        assert_eq!(document.line(2), Some(RopeSlice::from("")));

        let mut document = Document::from(Rect::default(), "hello".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(2));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor(), Cursor::new(2, 0));

        document.insert_newline();
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor(), Cursor::new(0, 1));
        assert_eq!(document.line(0), Some(RopeSlice::from("he")));
        assert_eq!(document.line(1), Some(RopeSlice::from("llo")));
    }

    #[test]
    fn document_move_cursor_to_line_start() {
        let mut document = Document::from(Rect::default(), "hello\nworld\n".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor(), Cursor::new(5, 0));
        document.move_cursor_to_line_start();
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(0, 2));
    }

    #[test]
    fn document_move_cursor_to_line_end() {
        let mut document = Document::from(Rect::default(), "hello\n\nworld".as_bytes()).unwrap();
        assert_eq!(document.cursor(), Cursor::new(0, 0));
        document.move_cursor_to_line_end();
        assert_eq!(document.cursor(), Cursor::new(5, 0));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor(), Cursor::new(5, 2));
    }

    #[test]
    fn document_line() {
        let document = Document::from(Rect::default(), "hello\n\nworld".as_bytes()).unwrap();
        assert_eq!(document.line(0).unwrap(), RopeSlice::from("hello"));
        assert_eq!(document.line(1).unwrap(), RopeSlice::from(""));
        assert_eq!(document.line(2).unwrap(), RopeSlice::from("world"));
        assert_eq!(document.line(3), None);
    }
}
