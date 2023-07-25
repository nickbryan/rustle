use crate::{row::Row, ui::Position};
use anyhow::{Error, Result};

pub struct Document {
    rows: Vec<Row>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            rows: vec![Row::default()],
        }
    }
}

impl Document {
    pub fn delete(&mut self, at: &Position) {
        if at.row >= self.len() {
            return;
        }

        if at.col == self.rows.get_mut(at.row).unwrap().len() && at.row < self.len() - 1 {
            let next_row = self.rows.remove(at.row + 1);
            let row = self.rows.get_mut(at.row).unwrap();
            row.append(&next_row);
            return;
        }

        let row = self.rows.get_mut(at.row).unwrap();
        row.delete(at.col);
    }

    pub fn insert(&mut self, at: &Position, ch: char) -> Result<()> {
        use std::cmp::Ordering;

        match at.row.cmp(&self.len()) {
            Ordering::Equal => {
                let mut row = Row::default();
                row.insert(0, ch);
                self.rows.push(row);

                Ok(())
            }
            Ordering::Less => {
                let row = self.rows.get_mut(at.row).unwrap();
                row.insert(at.col, ch);
                Ok(())
            }
            Ordering::Greater => Err(Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "trying to insert character past current string length",
            ))),
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.row > self.len() {
            return;
        }

        if at.row == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let new_row = self.rows.get_mut(at.row).unwrap().split(at.col);
        self.rows.insert(at.row + 1, new_row);
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}
