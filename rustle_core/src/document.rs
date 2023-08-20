use crate::render;
use crate::ui::Position;
use anyhow::{Context, Result};
use ropey::{Rope, RopeSlice};
use std::io::Read;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

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
    desired_col: usize,
}

impl Document {
    // TODO: test REsult
    pub fn from(reader: impl Read) -> Result<Self> {
        Ok(Self {
            text: Rope::from_reader(reader).context("creating rope from reader")?,
            selection: Selection::default(),
            desired_col: 0,
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
            .as_str()
            .unwrap()
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
            .as_str()
            .unwrap()
            .graphemes(true)
            .enumerate()
        {
            cursor += render::grapheme_width(grapheme);
        }

        self.desired_col = cursor;
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

                if self.desired_col > 0 {
                    let mut visual_width_moved = 0;
                    for (_, grapheme) in self
                        .text
                        .line(target_line)
                        .as_str()
                        .unwrap()
                        .graphemes(true)
                        .enumerate()
                    {
                        let grapheme_visual_width = render::grapheme_width(grapheme);
                        if visual_width_moved + grapheme_visual_width > self.desired_col {
                            break;
                        }

                        visual_width_moved += grapheme_visual_width;

                        cursor = nth_next_grapheme_boundary(self.text.slice(..), cursor, 1);

                        if self.desired_col < visual_width_moved {
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

                if self.desired_col > 0 {
                    let mut visual_width_moved = 0;
                    for (_, grapheme) in self
                        .text
                        .line(target_line)
                        .as_str()
                        .unwrap()
                        .graphemes(true)
                        .enumerate()
                    {
                        let grapheme_visual_width = render::grapheme_width(grapheme);
                        if visual_width_moved + grapheme_visual_width > self.desired_col {
                            break;
                        }

                        visual_width_moved += grapheme_visual_width;

                        cursor = nth_next_grapheme_boundary(self.text.slice(..), cursor, 1);

                        if self.desired_col < visual_width_moved {
                            cursor = nth_prev_grapheme_boundary(self.text.slice(..), cursor, 1);
                        }
                    }
                }

                self.selection.head = cursor.min(self.text.line_to_char(target_line + 1) - 1);
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

pub fn is_grapheme_boundary(slice: RopeSlice, char_idx: usize) -> bool {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it.
    let (chunk, chunk_byte_idx, _, _) = slice.chunk_at_byte(byte_idx);

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Determine if the given position is a grapheme cluster boundary.
    loop {
        match gc.is_boundary(chunk, chunk_byte_idx) {
            Ok(n) => return n,
            Err(GraphemeIncomplete::PreContext(n)) => {
                let (ctx_chunk, ctx_byte_start, _, _) = slice.chunk_at_byte(n - 1);
                gc.provide_context(ctx_chunk, ctx_byte_start);
            }
            Err(_) => unreachable!(),
        }
    }
}

pub fn nth_next_grapheme_boundary(slice: RopeSlice, char_idx: usize, n: usize) -> usize {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let mut byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it.
    let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = slice.chunk_at_byte(byte_idx);

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Find the nth next grapheme cluster boundary.
    for _ in 0..n {
        loop {
            match gc.next_boundary(chunk, chunk_byte_idx) {
                Ok(None) => return slice.len_chars(),
                Ok(Some(n)) => {
                    byte_idx = n;
                    break;
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    chunk_byte_idx += chunk.len();
                    let (a, _, c, _) = slice.chunk_at_byte(chunk_byte_idx);
                    chunk = a;
                    chunk_char_idx = c;
                }
                Err(GraphemeIncomplete::PreContext(n)) => {
                    let ctx_chunk = slice.chunk_at_byte(n - 1).0;
                    gc.provide_context(ctx_chunk, n - ctx_chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }
    let tmp = ropey::str_utils::byte_to_char_idx(chunk, byte_idx - chunk_byte_idx);
    chunk_char_idx + tmp
}

pub fn nth_prev_grapheme_boundary(slice: RopeSlice, char_idx: usize, n: usize) -> usize {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let mut byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it.
    let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = slice.chunk_at_byte(byte_idx);

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Find the previous grapheme cluster boundary.
    for _ in 0..n {
        loop {
            match gc.prev_boundary(chunk, chunk_byte_idx) {
                Ok(None) => return 0,
                Ok(Some(n)) => {
                    byte_idx = n;
                    break;
                }
                Err(GraphemeIncomplete::PrevChunk) => {
                    let (a, b, c, _) = slice.chunk_at_byte(chunk_byte_idx - 1);
                    chunk = a;
                    chunk_byte_idx = b;
                    chunk_char_idx = c;
                }
                Err(GraphemeIncomplete::PreContext(n)) => {
                    let ctx_chunk = slice.chunk_at_byte(n - 1).0;
                    gc.provide_context(ctx_chunk, n - ctx_chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }
    let tmp = ropey::str_utils::byte_to_char_idx(chunk, byte_idx - chunk_byte_idx);
    chunk_char_idx + tmp
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
    fn document_move_cursor_horizontally() {
        // Default position is zero for default Document.
        let mut document = Document::default();
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        // Moving forward does nothing in an empty Document.
        document.move_cursor_horizontally(Direction::Forward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Forward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        // Moving backward does nothing in an empty Document.
        document.move_cursor_horizontally(Direction::Backward(0));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        document.move_cursor_horizontally(Direction::Backward(10));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));

        // Default position is zero for new Document.
        let mut document = Document::from("1234\nabcd\n🇬🇧🇯🇲🇧🇪🏴󠁧󠁢󠁥󠁮󠁧󠁿\n🦀🌳🦀🌳\n".as_bytes()).unwrap();
        assert_eq!(document.cursor_coordinates(), Position::new(0, 0));
        // Moving forward.
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
        // Grapheme handling
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(2, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(4, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(6, 2));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(8, 2));
        // Ending
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 3));
        document.move_cursor_horizontally(Direction::Forward(5));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 4));
        document.move_cursor_horizontally(Direction::Forward(1));
        assert_eq!(document.cursor_coordinates(), Position::new(0, 4));

        // Moving backward.
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
}
