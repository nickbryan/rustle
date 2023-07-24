use unicode_segmentation::UnicodeSegmentation;

/// A single row of text within the editor.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Row {
    string: String,
}

impl Row {
    /// Convert the Row to a String allowing for a fixed length to be taken.
    pub fn to_string(&self, start: usize, end: usize) -> String {
        use std::cmp;

        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();

        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "\t" {
                result.push(' ');
            } else {
                result.push_str(grapheme);
            }
        }

        result
    }

    /// Convert the full Row to a String.
    pub fn contents(&self) -> String {
        self.to_string(0, self.len())
    }

    /// Append another Row to the current Row.
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
    }

    /// Delete the character at the given index. If the index is greater than the length of the
    /// Row then nothing will happen.
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }

        let mut result: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at + 1).collect();
        result.push_str(&remainder);
        self.string = result;
    }

    /// Insert a character at the given position in the Row. If the index is greater than the
    /// length of the Row then the character will be insterted at the next position.
    pub fn insert(&mut self, at: usize, ch: char) {
        if at >= self.len() {
            self.string.push(ch);
            return;
        }

        let mut result: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at).collect();

        result.push(ch);
        result.push_str(&remainder);
        self.string = result;
    }

    /// Split the Row at the given position, returning a new Row with the split string and updating
    /// the current row to the first half of the split.
    pub fn split(&mut self, at: usize) -> Self {
        let beginning: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at).collect();
        self.string = beginning;
        Self::from(&remainder[..])
    }

    /// The length of the Row. Graphemes are accounted for.
    pub fn len(&self) -> usize {
        self.string[..].graphemes(true).count()
    }

    /// Convert the Row to an array of bytes for writing.
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    /// Returns `true` if this `Row` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// This is the only way a Row can be constructed.
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Row;

    #[test]
    fn to_string_can_take_a_snippet_of_the_row() {
        assert_eq!("Hello", &Row::from("Hello World!").to_string(0, 5));
        assert_eq!("World", &Row::from("Hello World!").to_string(6, 11));
        assert_eq!(
            "\u{1f980}\u{1f980}\u{1f980}",
            &Row::from("\u{1f980}\u{1f980}\u{1f980}\u{1f980}\u{1f980}").to_string(1, 4)
        );
    }

    #[test]
    fn to_string_clamps_to_row_len() {
        assert_eq!("Hello", &Row::from("Hello").to_string(0, 100));
    }

    #[test]
    fn to_string_converts_tab_to_space() {
        assert_eq!("  ", &Row::from("\t\t\t\t").to_string(1, 3));
    }

    #[test]
    fn contents_returns_the_full_row() {
        assert_eq!(
            "Hello \u{1f980}!",
            &Row::from("Hello \u{1f980}!").contents()
        );
    }

    #[test]
    fn row_can_be_appended_to_another_row() {
        let mut row = Row::from("Beans");
        row.append(&Row::from(" on toast!"));
        assert_eq!("Beans on toast!", &row.contents());
    }

    #[test]
    fn character_can_be_deleted_at_position() {
        let mut row = Row::from("Beans");
        row.delete(1); // e
        row.delete(3); // s
        assert_eq!("Ban", &row.contents());
    }

    #[test]
    fn grapheme_can_be_deleted_at_position() {
        let mut row = Row::from("Crab\u{1f980}");
        row.delete(4);
        assert_eq!("Crab", &row.contents());
    }

    #[test]
    fn character_can_be_inserted_at_position() {
        let mut row = Row::from("13");
        row.insert(0, '0');
        row.insert(2, '2');
        row.insert(4, '4');
        assert_eq!("01234", &row.contents());
    }

    #[test]
    fn grapheme_can_be_inserted_at_position() {
        let mut row = Row::from("12");
        row.insert(1, '🦀');
        assert_eq!("1\u{1f980}2", &row.contents());
    }

    #[test]
    fn row_can_be_split_at_position() {
        let mut row = Row::from("\u{1f980} Rust is awesome!");
        let other = row.split(10);
        assert_eq!("\u{1f980} Rust is ", &row.contents());
        assert_eq!("awesome!", &other.contents());
    }

    #[test]
    fn len_is_calculated() {
        assert_eq!(12, Row::from("Hello World!").len());
    }

    #[test]
    fn len_counts_grapheme_clusters_individually() {
        assert_eq!(4, Row::from("\u{1f980}g\u{308}\u{ac01}\u{e01}").len());
    }

    #[test]
    fn row_can_be_converted_to_bytes_for_writing() {
        assert_eq!([72, 101, 108, 108, 111], Row::from("Hello").as_bytes());
    }

    #[test]
    fn is_empty_is_true_when_len_is_zero() {
        assert!(Row::default().is_empty());
    }

    #[test]
    fn is_empty_is_false_when_len_is_greater_than_zero() {
        assert!(!Row::from("123").is_empty());
    }
}
