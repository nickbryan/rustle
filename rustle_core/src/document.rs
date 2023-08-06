use crate::ui::Position;
use anyhow::Result;
use ropey::{Rope, RopeSlice};
use std::fs::File;
use std::io;

pub struct Document {
    text: Rope, // TODO: graphemes need handling
}

impl Default for Document {
    fn default() -> Self {
        Self { text: Rope::new() }
    }
}

impl Document {
    pub fn from_file(path: &str) -> Result<Self> {
        Ok(Self {
            text: Rope::from_reader(&mut io::BufReader::new(File::open(path)?))?,
        })
    }

    pub fn delete(&mut self, at: &Position) {
        if at.row >= self.len() {
            return;
        }

        self.text.remove(
            (self.text.line_to_char(at.row) + at.col)..=(self.text.line_to_char(at.row) + at.col),
        );
    }

    pub fn insert(&mut self, at: &Position, ch: char) {
        //TODO: handle bounds checks here
        self.text
            .insert_char(self.text.line_to_char(at.row) + at.col, ch);
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.row > self.len() {
            return;
        }

        self.text
            .insert_char(self.text.line_to_char(at.row) + at.col, '\n');
    }

    pub fn row(&self, index: usize) -> Option<RopeSlice> {
        if index >= self.text.len_lines() {
            return None;
        }

        self.text.get_line(index).map(|slice| {
            if slice.len_chars() > 0 && slice.char(slice.len_chars() - 1) == '\n' {
                slice.slice(0..slice.len_chars() - 1)
            } else {
                slice
            }
        })
    }

    pub fn len(&self) -> usize {
        self.text.len_lines()
    }
}
