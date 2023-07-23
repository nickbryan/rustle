/// Colors supported by the editor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    AnsiValue(u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::Reset
    }
}

/// A position in ui space.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    /// Create a new Position.
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
}

impl From<(usize, usize)> for Position {
    fn from((col, row): (usize, usize)) -> Self {
        Self::new(col, row)
    }
}

/// Rect represents an area/container in the ui.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Rect {
    pub width: usize,
    pub height: usize,
    pub position: Position,
}

impl Rect {
    /// Create a new Rect with default Position (0, 0).
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            position: Position::default(),
        }
    }

    /// Create a new Rect with a set Position.
    pub fn positioned(width: usize, height: usize, col: usize, row: usize) -> Self {
        Self {
            width,
            height,
            position: Position::new(col, row),
        }
    }

    /// Returns the area of the Rect.
    pub fn area(&self) -> usize {
        self.width.saturating_mul(self.height)
    }

    /// Returns the leftmost possible value of the Rect. **Note**: This is zero based.
    pub fn left(&self) -> usize {
        self.position.col
    }

    /// Returns the rightmost possible value of the Rect. **Note**: This is zero based.
    pub fn right(&self) -> usize {
        self.position.col + self.width - 1
    }

    /// Returns the topmost possible value of the Rect. **Note**: This is zero based.
    pub fn top(&self) -> usize {
        self.position.row
    }

    /// Returns the bottommost possible value of the Rect. **Note**: This is zero based.
    pub fn bottom(&self) -> usize {
        self.position.row + self.height - 1
    }

    /// Check if the given position is within the Rect, taking the Rect's Position into
    /// consideration.
    pub fn contains(&self, position: &Position) -> bool {
        let Position { col, row } = *position;

        col >= self.left() && col <= self.right() && row >= self.top() && row <= self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::{Position, Rect};

    #[test]
    fn new_sets_default_position() {
        let r = Rect::new(0, 0);
        assert_eq!(r.position.col, 0);
        assert_eq!(r.position.row, 0);
    }

    #[test]
    fn positioned_sets_position() {
        let r = Rect::positioned(0, 0, 10, 20);
        assert_eq!(r.position.col, 10);
        assert_eq!(r.position.row, 20);
    }

    #[test]
    fn area_is_calculated() {
        assert_eq!(Rect::new(10, 10).area(), 100);
    }

    #[test]
    fn left_returns_leftmost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).left(), 0);
    }

    #[test]
    fn left_returns_leftmost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 10, 0).left(), 10);
    }

    #[test]
    fn right_returns_rightmost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).right(), 4);
    }

    #[test]
    fn right_returns_rightmost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 20, 25).right(), 24);
    }

    #[test]
    fn top_returns_topmost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).top(), 0);
    }

    #[test]
    fn top_returns_topmost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 0, 12).top(), 12);
    }

    #[test]
    fn bottom_returns_bottommost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).bottom(), 9);
    }

    #[test]
    fn bottom_returns_bottommost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 20, 25).bottom(), 34);
    }

    #[test]
    fn contains_returns_true_if_position_contained() {
        let r = Rect::new(10, 10);
        assert!(r.contains(&Position::new(9, 9)));
    }

    #[test]
    fn contains_returns_false_if_position_not_contained() {
        let r = Rect::positioned(10, 10, 10, 10);
        assert_eq!(r.contains(&Position::new(20, 20)), false);
    }
}
