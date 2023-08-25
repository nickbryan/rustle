use crate::graphemes::{nth_next_grapheme_boundary, nth_prev_grapheme_boundary};
use crate::render;
use crate::ui::Position;
use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::io::Read;
use unicode_segmentation::UnicodeSegmentation;

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

#[derive(Debug, Default)]
pub(crate) struct Document {
    text: Rope, // TODO: graphemes need handling
    selection: Selection,
    desired_visual_col: usize,
}

impl Document {
    // TODO: test REsult
    pub(crate) fn from(reader: impl Read) -> Result<Self> {
        Ok(Self {
            text: Rope::from_reader(reader).context("creating rope from reader")?,
            selection: Selection::default(),
            desired_visual_col: 0,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.text.len_lines()
    }

    pub(crate) fn cursor_coordinates(&self) -> Position {
        let mut cursor = 0;

        for (_, grapheme) in self
            .text
            .slice(
                self.text
                    .line_to_char(self.text.char_to_line(self.selection.head))
                    ..self.selection.head,
            )
            .to_string()
            .graphemes(true)
            .enumerate()
        {
            cursor += render::grapheme_width(grapheme);
        }

        Position::new(cursor, self.text.char_to_line(self.selection.head))
    }

    pub(crate) fn move_cursor_horizontally(&mut self, direction: Direction) {
        match direction {
            Direction::Forward(chars) => {
                self.selection.head =
                    nth_next_grapheme_boundary(self.text.slice(..), self.selection.head, chars);
                self.selection.anchor =
                    nth_next_grapheme_boundary(self.text.slice(..), self.selection.anchor, chars);
            }
            Direction::Backward(chars) => {
                self.selection.head =
                    nth_prev_grapheme_boundary(self.text.slice(..), self.selection.head, chars);
                self.selection.anchor =
                    nth_prev_grapheme_boundary(self.text.slice(..), self.selection.anchor, chars);
            }
        };

        let mut cursor = 0;
        let current_line = self.text.char_to_line(self.selection.head);
        let current_line_start_idx = self.text.line_to_char(current_line);

        for (_, grapheme) in self
            .text
            .slice(current_line_start_idx..self.selection.head)
            .to_string()
            .graphemes(true)
            .enumerate()
        {
            cursor += render::grapheme_width(grapheme);
        }

        self.desired_visual_col = cursor;
    }

    pub(crate) fn move_cursor_vertically(&mut self, direction: Direction) {
        match direction {
            // Down.
            Direction::Forward(lines) => {
                let current_line = self.text.char_to_line(self.selection.head);
                let target_line = current_line
                    .saturating_add(lines)
                    .min(self.text.len_lines().saturating_sub(1));
                let target_line_start_idx = self.text.line_to_char(target_line);

                let mut cursor = target_line_start_idx;

                if self.desired_visual_col > 0 {
                    let mut visual_width_moved = 0;
                    for (_, grapheme) in self
                        .text
                        .line(target_line)
                        .to_string()
                        .graphemes(true)
                        .enumerate()
                    {
                        let grapheme_visual_width = render::grapheme_width(grapheme);
                        if visual_width_moved + grapheme_visual_width > self.desired_visual_col {
                            break;
                        }

                        visual_width_moved += grapheme_visual_width;

                        cursor = nth_next_grapheme_boundary(self.text.slice(..), cursor, 1);

                        if self.desired_visual_col < visual_width_moved {
                            cursor = nth_prev_grapheme_boundary(self.text.slice(..), cursor, 1);
                        }
                    }
                }

                self.selection.head = cursor;

                if target_line.saturating_add(1) < self.text.len_lines() {
                    self.selection.head = self.selection.head.min(
                        self.text
                            .line_to_char(target_line.saturating_add(1))
                            .saturating_sub(1),
                    );
                }
                self.selection.anchor = self.selection.head;
            }
            // Up
            Direction::Backward(lines) => {
                let current_line = self.text.char_to_line(self.selection.head);
                let target_line = current_line.saturating_sub(lines);
                let target_line_start_idx = self.text.line_to_char(target_line);

                let mut cursor = target_line_start_idx;

                if self.desired_visual_col > 0 {
                    let mut visual_width_moved = 0;
                    for (_, grapheme) in self
                        .text
                        .line(target_line)
                        .to_string()
                        .graphemes(true)
                        .enumerate()
                    {
                        let grapheme_visual_width = render::grapheme_width(grapheme);
                        if visual_width_moved + grapheme_visual_width > self.desired_visual_col {
                            break;
                        }

                        visual_width_moved += grapheme_visual_width;

                        cursor = nth_next_grapheme_boundary(self.text.slice(..), cursor, 1);

                        if self.desired_visual_col < visual_width_moved {
                            cursor = nth_prev_grapheme_boundary(self.text.slice(..), cursor, 1);
                        }
                    }
                }

                self.selection.head = cursor.min(
                    self.text
                        .line_to_char(target_line.saturating_add(1))
                        .saturating_sub(1),
                );
                self.selection.anchor = self.selection.head;
            }
        };
    }

    pub(crate) fn move_cursor_to_line_start(&mut self) {
        self.selection.head = self
            .text
            .line_to_char(self.text.char_to_line(self.selection.head));
        self.selection.anchor = self.selection.head;

        let mut cursor = 0;
        let current_line = self.text.char_to_line(self.selection.head);
        let current_line_start_idx = self.text.line_to_char(current_line);

        for (_, grapheme) in self
            .text
            .slice(current_line_start_idx..self.selection.head)
            .to_string()
            .graphemes(true)
            .enumerate()
        {
            cursor += render::grapheme_width(grapheme);
        }

        self.desired_visual_col = cursor;
    }

    pub(crate) fn move_cursor_to_line_end(&mut self) {
        self.selection.head = nth_prev_grapheme_boundary(
            self.text.slice(..),
            self.text
                .line_to_char(self.text.char_to_line(self.selection.head) + 1),
            1,
        );
        self.selection.anchor = self.selection.head;

        let mut cursor = 0;
        let current_line = self.text.char_to_line(self.selection.head);
        let current_line_start_idx = self.text.line_to_char(current_line);

        for (_, grapheme) in self
            .text
            .slice(current_line_start_idx..self.selection.head)
            .to_string()
            .graphemes(true)
            .enumerate()
        {
            cursor += render::grapheme_width(grapheme);
        }

        self.desired_visual_col = cursor;
    }

    pub(crate) fn delete(&mut self, direction: Direction) {
        match direction {
            Direction::Forward(chars) => self.text.remove(
                self.selection.head
                    ..nth_next_grapheme_boundary(self.text.slice(..), self.selection.head, chars),
            ),
            Direction::Backward(chars) => {
                let start = self.selection.head;
                self.move_cursor_horizontally(Direction::Backward(chars));
                self.text.remove(self.selection.head..start);
            }
        };
    }

    pub(crate) fn insert(&mut self, ch: char) {
        self.text.insert_char(self.selection.head, ch);
        self.move_cursor_horizontally(Direction::Forward(1));
    }

    pub(crate) fn insert_newline(&mut self) {
        self.insert('\n');
    }

    // TODO: test
    pub(crate) fn line(&self, line_number: usize) -> Option<RopeSlice> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Position;

    struct FileNotFoundReader;

    impl Read for FileNotFoundReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }

    #[test]
    fn document_from_reader_handles_error_with_context() {
        let error = Document::from(FileNotFoundReader).unwrap_err();

        assert_eq!(
            Document::from(FileNotFoundReader).unwrap_err().to_string(),
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
        assert_eq!(Document::from("1".as_bytes()).unwrap().len(), 1);
        assert_eq!(Document::from("1\n".as_bytes()).unwrap().len(), 2);
        assert_eq!(Document::from("1\n2\n3\n".as_bytes()).unwrap().len(), 4);
    }

    #[test]
    fn document_move_cursor_horizontally_does_nothing_for_empty_document() {
        let mut document = Document::default();
        document.move_cursor_horizontally(Direction::Forward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
    }

    #[test]
    fn document_move_cursor_horizontally_through_document() {
        let mut document = Document::from("1234\nabcd\n🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n🦀🌳🦀🌳\n".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(2, 0));
        document.move_cursor_horizontally(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(4, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 1));
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(2, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(4, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(6, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 3));
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 4));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 4));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 3));
        document.move_cursor_horizontally(Direction::Backward(4));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 3));
        document.move_cursor_horizontally(Direction::Backward(5));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));
        document.move_cursor_horizontally(Direction::Backward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
    }

    #[test]
    fn document_move_cursor_vertically_does_nothing_for_empty_document() {
        let mut document = Document::default();
        document.move_cursor_vertically(Direction::Forward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Backward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Backward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
    }

    #[test]
    fn document_move_cursor_vertically_through_first_column() {
        let mut document = Document::from(
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
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 1));
        document.move_cursor_vertically(Direction::Forward(100));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 7));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 6));
        document.move_cursor_vertically(Direction::Backward(100));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
    }

    #[test]
    fn document_move_cursor_vertically_through_second_column() {
        let mut document = Document::from(
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
        assert_eq!(document.cursor_coordinates(), Position::new(1, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 1));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 3));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 4));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 5));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 6));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 7));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 7));
        document.move_cursor_vertically(Direction::Backward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 5));
        document.move_cursor_vertically(Direction::Backward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 1));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 0));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 0));
    }

    #[test]
    fn document_move_cursor_vertically_aligns_to_previous_grapheme_boundary() {
        let mut document = Document::from(
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
        assert_eq!(document.cursor_coordinates(), Position::new(7, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(6, 1));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(6, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(7, 2));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(6, 1));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(6, 3));
        document.move_cursor_vertically(Direction::Backward(3));
        assert_eq!(document.cursor_coordinates(), Position::new(7, 0));
    }

    #[test]
    fn document_move_cursor_vertically_aligns_to_next_grapheme_boundary() {
        // TODO: these tests, make them cleaner
        let mut document = Document::from(
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
        assert_eq!(document.cursor_coordinates(), Position::new(8, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 1));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 2));
    }

    #[test]
    fn document_move_cursor_vertically_handles_different_line_lengths() {
        // TODO: these tests, make them cleaner
        let mut document = Document::from(
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
        assert_eq!(document.cursor_coordinates(), Position::new(26, 0));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 1));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(14, 2));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 3));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(26, 4));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 5));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 6));
        document.move_cursor_vertically(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(26, 7));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 6));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 5));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(26, 4));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(1, 3));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(14, 2));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 1));
        document.move_cursor_vertically(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(26, 0));
    }

    #[test]
    fn document_delete_forward() {
        let mut document = Document::from(
            "\
                        abc\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));

        document.delete(Direction::Forward(1));
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("bc")));

        document.delete(Direction::Forward(2));
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("")));

        document.delete(Direction::Forward(1));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿")));

        document.delete(Direction::Forward(1));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿")));

        document.delete(Direction::Forward(100));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("")));
    }

    #[test]
    fn document_delete_backward() {
        let mut document = Document::from(
            "\
                        abc\n\
                        🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n\
                    "
            .as_bytes(),
        )
        .unwrap();
        document.move_cursor_horizontally(Direction::Forward(9));
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));

        document.delete(Direction::Backward(1));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(8, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿")));

        document.delete(Direction::Backward(2));
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(4, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬🇧🇯🇲")));

        document.delete(Direction::Backward(100));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("")));
    }

    #[test]
    fn document_insert() {
        let mut document = Document::default();
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));

        document.insert('h');
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(1, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("h")));
        document.insert('3');
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(2, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("h3")));
        document.insert('\n');
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 1));
        assert_eq!(document.line(0), Some(RopeSlice::from("h3")));
        assert_eq!(document.line(1), Some(RopeSlice::from("")));
        document.insert('🇬');
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(1, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬")));
        document.insert('🇧');
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(2, 1));
        assert_eq!(document.line(1), Some(RopeSlice::from("🇬🇧")));

        let mut document = Document::from("ello".as_bytes()).unwrap();
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));

        document.insert('h');
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(1, 0));
        assert_eq!(document.line(0), Some(RopeSlice::from("hello")));
    }

    #[test]
    fn document_insert_newline() {
        let mut document = Document::from("hello".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(100));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(5, 0));

        document.insert_newline();
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 1));
        assert_eq!(document.line(0), Some(RopeSlice::from("hello")));
        assert_eq!(document.line(1), Some(RopeSlice::from("")));
        document.insert_newline();
        assert_eq!(document.len(), 3);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));
        assert_eq!(document.line(0), Some(RopeSlice::from("hello")));
        assert_eq!(document.line(1), Some(RopeSlice::from("")));
        assert_eq!(document.line(2), Some(RopeSlice::from("")));

        let mut document = Document::from("hello".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(2));
        assert_eq!(document.len(), 1);
        assert_eq!(document.cursor_coordinates(), Position::new(2, 0));

        document.insert_newline();
        assert_eq!(document.len(), 2);
        assert_eq!(document.cursor_coordinates(), Position::new(0, 1));
        assert_eq!(document.line(0), Some(RopeSlice::from("he")));
        assert_eq!(document.line(1), Some(RopeSlice::from("llo")));
    }

    #[test]
    fn document_move_cursor_to_line_start() {
        let mut document = Document::from("hello\nworld\n".as_bytes()).unwrap();
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor_coordinates(), Position::new(5, 0));
        document.move_cursor_to_line_start();
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 2));
    }

    #[test]
    fn document_move_cursor_to_line_end() {
        let mut document = Document::from("hello\n\nworld".as_bytes()).unwrap();
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_to_line_end();
        assert_eq!(document.cursor_coordinates(), Position::new(5, 0));
        document.move_cursor_vertically(Direction::Forward(2));
        assert_eq!(document.cursor_coordinates(), Position::new(5, 2));
    }
}
