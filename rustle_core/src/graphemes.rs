use crate::render;
use ropey::Rope;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

pub trait RopeExt {
    fn next_grapheme_boundary(&self, char_idx: usize) -> usize;
    fn nth_next_grapheme_boundary(&self, char_idx: usize, n: usize) -> usize;
    fn nth_prev_grapheme_boundary(&self, char_idx: usize, n: usize) -> usize;
    fn prev_grapheme_boundary(&self, char_idx: usize) -> usize;
    fn visual_column_position(&self, char_idx: usize) -> usize;
    fn visual_column_position_to_char_idx(&self, char_idx: usize, col_pos_idx: usize) -> usize;
}

impl RopeExt for Rope {
    fn next_grapheme_boundary(&self, char_idx: usize) -> usize {
        self.nth_next_grapheme_boundary(char_idx, 1)
    }

    fn nth_next_grapheme_boundary(&self, char_idx: usize, n: usize) -> usize {
        // Bounds check
        debug_assert!(char_idx <= self.len_chars());

        // We work with bytes for this, so convert.
        let mut byte_idx = self.char_to_byte(char_idx);

        // Get the chunk with our byte index in it.
        let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = self.chunk_at_byte(byte_idx);

        // Set up the grapheme cursor.
        let mut gc = GraphemeCursor::new(byte_idx, self.len_bytes(), true);

        // Find the nth next grapheme cluster boundary.
        for _ in 0..n {
            loop {
                match gc.next_boundary(chunk, chunk_byte_idx) {
                    Ok(None) => return self.len_chars(),
                    Ok(Some(n)) => {
                        byte_idx = n;
                        break;
                    }
                    Err(GraphemeIncomplete::NextChunk) => {
                        chunk_byte_idx += chunk.len();
                        let (a, _, c, _) = self.chunk_at_byte(chunk_byte_idx);
                        chunk = a;
                        chunk_char_idx = c;
                    }
                    Err(GraphemeIncomplete::PreContext(n)) => {
                        let ctx_chunk = self.chunk_at_byte(n - 1).0;
                        gc.provide_context(ctx_chunk, n - ctx_chunk.len());
                    }
                    _ => unreachable!(),
                }
            }
        }
        let tmp = ropey::str_utils::byte_to_char_idx(chunk, byte_idx - chunk_byte_idx);
        chunk_char_idx + tmp
    }

    fn nth_prev_grapheme_boundary(&self, char_idx: usize, n: usize) -> usize {
        // Bounds check
        debug_assert!(char_idx <= self.len_chars());

        // We work with bytes for this, so convert.
        let mut byte_idx = self.char_to_byte(char_idx);

        // Get the chunk with our byte index in it.
        let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = self.chunk_at_byte(byte_idx);

        // Set up the grapheme cursor.
        let mut gc = GraphemeCursor::new(byte_idx, self.len_bytes(), true);

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
                        let (a, b, c, _) = self.chunk_at_byte(chunk_byte_idx - 1);
                        chunk = a;
                        chunk_byte_idx = b;
                        chunk_char_idx = c;
                    }
                    Err(GraphemeIncomplete::PreContext(n)) => {
                        let ctx_chunk = self.chunk_at_byte(n - 1).0;
                        gc.provide_context(ctx_chunk, n - ctx_chunk.len());
                    }
                    _ => unreachable!(),
                }
            }
        }
        let tmp = ropey::str_utils::byte_to_char_idx(chunk, byte_idx - chunk_byte_idx);
        chunk_char_idx + tmp
    }

    fn prev_grapheme_boundary(&self, char_idx: usize) -> usize {
        self.nth_prev_grapheme_boundary(char_idx, 1)
    }

    fn visual_column_position(&self, char_idx: usize) -> usize {
        let mut column = 0;

        for (_, grapheme) in self
            .slice(self.line_to_char(self.char_to_line(char_idx))..char_idx)
            .to_string()
            .graphemes(true)
            .enumerate()
        {
            column += render::grapheme_width(grapheme);
        }

        column
    }

    fn visual_column_position_to_char_idx(&self, line_idx: usize, col_pos_idx: usize) -> usize {
        let mut cursor = self.line_to_char(line_idx);
        let mut visual_width_moved = 0;
        for (_, grapheme) in self.line(line_idx).to_string().graphemes(true).enumerate() {
            let grapheme_visual_width = render::grapheme_width(grapheme);
            if visual_width_moved + grapheme_visual_width > col_pos_idx {
                break;
            }

            visual_width_moved += grapheme_visual_width;

            cursor = self.next_grapheme_boundary(cursor);

            if col_pos_idx < visual_width_moved {
                cursor = self.prev_grapheme_boundary(cursor);
            }
        }
        cursor
    }
}
