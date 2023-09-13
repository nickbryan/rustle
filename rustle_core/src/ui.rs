use taffy::{geometry::Point, layout::Layout};

/// Colors supported by the editor.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Color {
    #[default]
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

/// A position in ui space.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}

impl Position {
    /// Create a new Position.
    #[must_use]
    pub fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

impl From<Point<f32>> for Position {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn from(point: Point<f32>) -> Self {
        Self::new(point.x as u16, point.y as u16)
    }
}

/// Rect represents an area/container in the ui.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Rect {
    pub width: u16,
    pub height: u16,
    pub position: Position,
}

impl Rect {
    /// Create a new Rect with default Position (0, 0).
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            position: Position::default(),
        }
    }

    /// Create a new Rect with a set Position.
    #[must_use]
    pub fn positioned(width: u16, height: u16, col: u16, row: u16) -> Self {
        Self {
            width,
            height,
            position: Position::new(col, row),
        }
    }

    /// Returns the area of the Rect.
    #[must_use]
    pub fn area(&self) -> usize {
        self.width.saturating_mul(self.height).into()
    }

    /// Returns the leftmost possible value of the Rect. **Note**: This is zero based.
    #[must_use]
    pub fn left(&self) -> u16 {
        self.position.col
    }

    /// Returns the rightmost possible value of the Rect. **Note**: This is zero based.
    #[must_use]
    pub fn right(&self) -> u16 {
        self.position.col + self.width - 1
    }

    /// Returns the topmost possible value of the Rect. **Note**: This is zero based.
    #[must_use]
    pub fn top(&self) -> u16 {
        self.position.row
    }

    /// Returns the bottommost possible value of the Rect. **Note**: This is zero based.
    #[must_use]
    pub fn bottom(&self) -> u16 {
        self.position.row + self.height - 1
    }

    /// Check if the given position is within the Rect, taking the Rect's Position into
    /// consideration.
    #[must_use]
    pub fn contains(&self, position: Position) -> bool {
        let Position { col, row } = position;

        col >= self.left() && col <= self.right() && row >= self.top() && row <= self.bottom()
    }
}

impl From<&Layout> for Rect {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn from(layout: &Layout) -> Self {
        Rect::positioned(
            layout.size.width as u16,
            layout.size.height as u16,
            layout.location.x as u16,
            layout.location.y as u16,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::geometry::Point;
    use taffy::layout::Layout;
    use taffy::prelude::Size;

    #[test]
    fn position_from_point_sets_col_and_row() {
        let p = Position::from(Point { x: 1.0, y: 2.0 });
        assert_eq!(p.col, 1);
        assert_eq!(p.row, 2);
    }

    #[test]
    fn rect_new_sets_default_position() {
        let r = Rect::new(0, 0);
        assert_eq!(r.position.col, 0);
        assert_eq!(r.position.row, 0);
    }

    #[test]
    fn rect_positioned_sets_position() {
        let r = Rect::positioned(0, 0, 10, 20);
        assert_eq!(r.position.col, 10);
        assert_eq!(r.position.row, 20);
    }

    #[test]
    fn rect_from_layout_sets_col_and_row() {
        let mut layout = Layout::new();
        layout.size = Size {
            width: 100.0,
            height: 200.0,
        };
        layout.location = Point { x: 3.0, y: 5.0 };
        assert_eq!(Rect::from(&layout), Rect::positioned(100, 200, 3, 5));
    }

    #[test]
    fn rect_area_is_calculated() {
        assert_eq!(Rect::new(10, 10).area(), 100);
    }

    #[test]
    fn rect_left_returns_leftmost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).left(), 0);
    }

    #[test]
    fn rect_left_returns_leftmost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 10, 0).left(), 10);
    }

    #[test]
    fn rect_right_returns_rightmost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).right(), 4);
    }

    #[test]
    fn rect_right_returns_rightmost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 20, 25).right(), 24);
    }

    #[test]
    fn rect_top_returns_topmost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).top(), 0);
    }

    #[test]
    fn rect_top_returns_topmost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 0, 12).top(), 12);
    }

    #[test]
    fn rect_bottom_returns_bottommost_possible_value() {
        assert_eq!(Rect::positioned(5, 10, 0, 0).bottom(), 9);
    }

    #[test]
    fn rect_bottom_returns_bottommost_possible_value_including_offset() {
        assert_eq!(Rect::positioned(5, 10, 20, 25).bottom(), 34);
    }

    #[test]
    fn rect_contains_returns_true_if_position_contained() {
        let r = Rect::new(10, 10);
        assert!(r.contains(Position::new(9, 9)));
    }

    #[test]
    fn rect_contains_returns_false_if_position_not_contained() {
        let r = Rect::new(10, 10);
        assert!(!r.contains(Position::new(10, 9)));
        assert!(!r.contains(Position::new(9, 10)));

        let r = Rect::positioned(10, 10, 10, 10);
        assert!(!r.contains(Position::new(20, 20)));
    }
}
