use crate::{row::Row, ui::Position};
use anyhow::{Error, Result};

pub struct Document {
    file_name: Option<String>,
    rows: Vec<Row>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            file_name: None,
            rows: vec![Row::default()],
        }
    }
}

impl Document {
    pub fn open(filename: &str) -> Result<Self> {
        use anyhow::Context;
        use std::fs;

        let contents = fs::read_to_string(filename).context("unable to read from file")?;
        let mut rows = Vec::new();

        for row in contents.lines() {
            rows.push(Row::from(row));
        }

        Ok(Self {
            file_name: Some(String::from(filename)),
            rows,
        })
    }

    pub fn save(&mut self, filename: Option<&str>) -> Result<(), std::io::Error> {
        use {std::fs::File, std::io::Write};

        if let Some(filename) = filename {
            self.file_name = Some(filename.into());
        }

        if let Some(file_name) = &self.file_name {
            let mut file = File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }

        Ok(())
    }

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

    pub fn file_name(&self) -> Option<&String> {
        self.file_name.as_ref()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}
