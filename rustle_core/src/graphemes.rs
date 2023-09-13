use ropey::Rope;
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete, UnicodeSegmentation};

use crate::render;

pub(crate) trait RopeExt {
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
        debug_assert!(
            char_idx <= self.len_chars(),
            "char_idx={} is out of bounds len={}",
            char_idx,
            self.len_chars()
        );

        let mut byte_idx = self.char_to_byte(char_idx);
        let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = self.chunk_at_byte(byte_idx);
        let mut gc = GraphemeCursor::new(byte_idx, self.len_bytes(), true);

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
        debug_assert!(
            char_idx <= self.len_chars(),
            "char_idx={} is out of bounds len={}",
            char_idx,
            self.len_chars()
        );

        let mut byte_idx = self.char_to_byte(char_idx);
        let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = self.chunk_at_byte(byte_idx);
        let mut gc = GraphemeCursor::new(byte_idx, self.len_bytes(), true);

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
        debug_assert!(
            char_idx <= self.len_chars(),
            "char_idx={} is out of bounds len={}",
            char_idx,
            self.len_chars()
        );

        self.slice(self.line_to_char(self.char_to_line(char_idx))..char_idx)
            .to_string()
            .graphemes(true)
            .map(render::grapheme_width)
            .sum()
    }

    fn visual_column_position_to_char_idx(&self, line_idx: usize, col_pos_idx: usize) -> usize {
        debug_assert!(
            line_idx <= self.len_lines(),
            "line_idx={} is out of bounds len_lines={}",
            line_idx,
            self.len_lines(),
        );

        let mut cursor = self.line_to_char(line_idx);
        let mut visual_width_moved = 0;

        for (_, grapheme) in self.line(line_idx).to_string().graphemes(true).enumerate() {
            let grapheme_visual_width = render::grapheme_width(grapheme);

            if visual_width_moved + grapheme_visual_width > col_pos_idx {
                break;
            }

            visual_width_moved += grapheme_visual_width;
            cursor = self.next_grapheme_boundary(cursor);
        }

        cursor
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::*;

    #[test]
    #[should_panic(expected = "char_idx=100 is out of bounds len=2")]
    fn ropeext_next_grapheme_boundary_bounds_check() {
        let text = Rope::from("ab");
        text.next_grapheme_boundary(100);
    }

    #[test]
    fn ropeext_next_grapheme_boundary() {
        let text = Rope::from("ab🇬🇧🏴󠁧󠁢󠁥󠁮󠁧󠁿\na");
        assert_eq!(text.next_grapheme_boundary(0), 1);
        assert_eq!(text.next_grapheme_boundary(1), 2);
        assert_eq!(text.next_grapheme_boundary(2), 4);
        assert_eq!(text.next_grapheme_boundary(3), 4);
        assert_eq!(text.next_grapheme_boundary(4), 11);
        assert_eq!(text.next_grapheme_boundary(5), 11);
        assert_eq!(text.next_grapheme_boundary(6), 11);
        assert_eq!(text.next_grapheme_boundary(7), 11);
        assert_eq!(text.next_grapheme_boundary(8), 11);
        assert_eq!(text.next_grapheme_boundary(9), 11);
        assert_eq!(text.next_grapheme_boundary(10), 11);
        assert_eq!(text.next_grapheme_boundary(11), 12);
        assert_eq!(text.next_grapheme_boundary(12), 13);
        assert_eq!(text.next_grapheme_boundary(13), 13);
    }

    #[test]
    #[should_panic(expected = "char_idx=100 is out of bounds len=2")]
    fn ropeext_nth_next_grapheme_boundary_bounds_check() {
        let text = Rope::from("ab");
        text.nth_next_grapheme_boundary(100, 10);
    }

    #[test]
    fn ropeext_nth_next_grapheme_boundary() {
        let text = Rope::from("ab🇬🇧🏴󠁧󠁢󠁥󠁮󠁧󠁿\na");
        assert_eq!(text.nth_next_grapheme_boundary(0, 1), 1);
        assert_eq!(text.nth_next_grapheme_boundary(0, 2), 2);
        assert_eq!(text.nth_next_grapheme_boundary(1, 2), 4);
        assert_eq!(text.nth_next_grapheme_boundary(3, 2), 11);
        assert_eq!(text.nth_next_grapheme_boundary(11, 1), 12);
        assert_eq!(text.nth_next_grapheme_boundary(12, 1), 13);
        assert_eq!(text.nth_next_grapheme_boundary(13, 1), 13);
    }

    #[test]
    #[should_panic(expected = "char_idx=100 is out of bounds len=2")]
    fn ropeext_nth_previous_grapheme_boundary_bounds_check() {
        let text = Rope::from("ab");
        text.nth_prev_grapheme_boundary(100, 10);
    }

    #[test]
    fn ropeext_nth_previous_grapheme_boundary() {
        let text = Rope::from("ab🇬🇧🏴󠁧󠁢󠁥󠁮󠁧󠁿\na");
        assert_eq!(text.nth_prev_grapheme_boundary(13, 1), 12);
        assert_eq!(text.nth_prev_grapheme_boundary(12, 1), 11);
        assert_eq!(text.nth_prev_grapheme_boundary(11, 3), 1);
        assert_eq!(text.nth_prev_grapheme_boundary(11, 2), 2);
        assert_eq!(text.nth_prev_grapheme_boundary(11, 1), 4);
        assert_eq!(text.nth_prev_grapheme_boundary(4, 2), 1);
        assert_eq!(text.nth_prev_grapheme_boundary(3, 2), 1);
        assert_eq!(text.nth_prev_grapheme_boundary(2, 1), 1);
        assert_eq!(text.nth_prev_grapheme_boundary(0, 100), 0);
    }

    #[test]
    #[should_panic(expected = "char_idx=100 is out of bounds len=2")]
    fn ropeext_previous_grapheme_boundary_bounds_check() {
        let text = Rope::from("ab");
        text.prev_grapheme_boundary(100);
    }

    #[test]
    fn ropeext_previous_grapheme_boundary() {
        let text = Rope::from("ab🇬🇧🏴󠁧󠁢󠁥󠁮󠁧󠁿\na");
        assert_eq!(text.prev_grapheme_boundary(13), 12);
        assert_eq!(text.prev_grapheme_boundary(12), 11);
        assert_eq!(text.prev_grapheme_boundary(11), 4);
        assert_eq!(text.prev_grapheme_boundary(10), 4);
        assert_eq!(text.prev_grapheme_boundary(9), 4);
        assert_eq!(text.prev_grapheme_boundary(8), 4);
        assert_eq!(text.prev_grapheme_boundary(7), 4);
        assert_eq!(text.prev_grapheme_boundary(6), 4);
        assert_eq!(text.prev_grapheme_boundary(5), 4);
        assert_eq!(text.prev_grapheme_boundary(4), 2);
        assert_eq!(text.prev_grapheme_boundary(3), 2);
        assert_eq!(text.prev_grapheme_boundary(2), 1);
        assert_eq!(text.prev_grapheme_boundary(1), 0);
        assert_eq!(text.prev_grapheme_boundary(0), 0);
    }

    #[test]
    #[should_panic(expected = "char_idx=100 is out of bounds len=2")]
    fn ropeext_visual_column_position_bounds_check() {
        let text = Rope::from("ab");
        text.visual_column_position(100);
    }

    #[test]
    fn ropeext_visual_column_position() {
        let text = Rope::from("ab🇬🇧🏴󠁧󠁢󠁥󠁮󠁧󠁿\na");
        assert_eq!(text.visual_column_position(0), 0);
        assert_eq!(text.visual_column_position(1), 1);
        assert_eq!(text.visual_column_position(2), 2);
        assert_eq!(text.visual_column_position(3), 3);
        assert_eq!(text.visual_column_position(4), 4);
        assert_eq!(text.visual_column_position(5), 6);
        assert_eq!(text.visual_column_position(6), 6);
        assert_eq!(text.visual_column_position(7), 6);
        assert_eq!(text.visual_column_position(8), 6);
        assert_eq!(text.visual_column_position(9), 6);
        assert_eq!(text.visual_column_position(10), 6);
        assert_eq!(text.visual_column_position(11), 6);
        assert_eq!(text.visual_column_position(12), 0); // Next line
        assert_eq!(text.visual_column_position(13), 1);
    }

    #[test]
    #[should_panic(expected = "line_idx=100 is out of bounds len_lines=1")]
    fn ropeext_visual_column_position_to_char_idx_lines_bounds_check() {
        let text = Rope::from("ab");
        text.visual_column_position_to_char_idx(100, 1);
    }

    #[test]
    fn ropeext_visual_column_position_to_char_idx() {
        let text = Rope::from(
            "\
        hello\n\
        🏴󠁧󠁢󠁥󠁮󠁧󠁿👩‍🔬\n\
        🇬🇧\n\
        ",
        );
        assert_eq!(text.visual_column_position_to_char_idx(0, 0), 0);
        assert_eq!(text.visual_column_position_to_char_idx(0, 1), 1);
        assert_eq!(text.visual_column_position_to_char_idx(0, 2), 2);
        assert_eq!(text.visual_column_position_to_char_idx(0, 3), 3);
        assert_eq!(text.visual_column_position_to_char_idx(0, 4), 4);
        assert_eq!(text.visual_column_position_to_char_idx(0, 5), 5);
        assert_eq!(text.visual_column_position_to_char_idx(0, 6), 6);
        // Caps to end of line.
        assert_eq!(text.visual_column_position_to_char_idx(0, 100), 6);

        assert_eq!(text.visual_column_position_to_char_idx(1, 0), 6);
        assert_eq!(text.visual_column_position_to_char_idx(1, 1), 6);
        assert_eq!(text.visual_column_position_to_char_idx(1, 2), 13);
        assert_eq!(text.visual_column_position_to_char_idx(1, 3), 13);
        assert_eq!(text.visual_column_position_to_char_idx(1, 4), 13);
        assert_eq!(text.visual_column_position_to_char_idx(1, 5), 13);
        assert_eq!(text.visual_column_position_to_char_idx(1, 6), 16);
        assert_eq!(text.visual_column_position_to_char_idx(1, 7), 17);
        assert_eq!(text.visual_column_position_to_char_idx(2, 0), 17);
        assert_eq!(text.visual_column_position_to_char_idx(2, 1), 17);
        assert_eq!(text.visual_column_position_to_char_idx(2, 2), 19);
        assert_eq!(text.visual_column_position_to_char_idx(2, 3), 20);
        assert_eq!(text.visual_column_position_to_char_idx(3, 0), 20);
    }
}
