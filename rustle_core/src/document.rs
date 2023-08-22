use crate::graphemes::{nth_next_grapheme_boundary, nth_prev_grapheme_boundary};
use crate::render;
use crate::ui::Position;
use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::io::Read;
use unicode_segmentation::UnicodeSegmentation;

pub enum Direction {
    Forward(usize),
    Backward(usize),
}

#[derive(Default)]
pub struct Selection {
    anchor: usize,
    head: usize,
}

#[derive(Default)]
pub struct Document {
    text: Rope, // TODO: graphemes need handling
    selection: Selection,
    desired_visual_col: usize,
}

impl Document {
    // TODO: test REsult
    pub fn from(reader: impl Read) -> Result<Self> {
        Ok(Self {
            text: Rope::from_reader(reader).context("creating rope from reader")?,
            selection: Selection::default(),
            desired_visual_col: 0,
        })
    }

    pub fn len(&self) -> usize {
        self.text.len_lines()
    }

    pub fn cursor_coordinates(&self) -> Position {
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

    pub fn move_cursor_horizontally(&mut self, direction: Direction) {
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

    pub fn move_cursor_vertically(&mut self, direction: Direction) {
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

    // TODO: test
    pub fn delete(&mut self, at: &Position) {
        if at.row >= self.len() {
            return;
        }

        self.text
            .remove(self.selection.anchor..=self.selection.head);
    }

    // TODO: test
    pub fn insert(&mut self, ch: char) {
        //TODO: handle bounds checks here
        self.text.insert_char(self.selection.head, ch);
    }

    // TODO: test
    pub fn insert_newline(&mut self) {
        self.insert('\n');
    }

    // TODO: test
    pub fn line(&self, line_number: usize) -> Option<RopeSlice> {
        if line_number >= self.text.len_lines() {
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
    fn document_move_cursor_vertically_aligns_to_previous_grapheme_boundry() {
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
    fn document_move_cursor_vertically_aligns_to_next_grapheme_boundry() {
        // TODO: these tests, make them cleaner
    }
}
