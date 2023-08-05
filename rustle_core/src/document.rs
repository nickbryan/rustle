use crate::ui::Position;
use crop::{Rope, RopeSlice};

pub struct Document {
    rope: Rope,
}

impl Default for Document {
    fn default() -> Self {
        Self { rope: Rope::new() }
    }
}

impl Document {
    pub fn delete(&mut self, at: &Position) {
        if at.row >= self.len() {
            return;
        }

        self.rope.delete(
            (self.rope.byte_of_line(at.row) + at.col)..=(self.rope.byte_of_line(at.row) + at.col),
        );
    }

    pub fn insert(&mut self, at: &Position, ch: char) {
        //TODO: handle bounds checks here
        self.rope
            .insert(self.rope.byte_of_line(at.row) + at.col, ch.to_string());
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.row > self.len() {
            return;
        }

        self.rope
            .insert(self.rope.byte_of_line(at.row) + at.col, "\n");
    }

    pub fn row(&self, index: usize) -> Option<RopeSlice> {
        if index >= self.rope.line_len() {
            return None;
        }

        Some(self.rope.line(index))
    }

    pub fn len(&self) -> usize {
        self.rope.line_len()
    }
}
