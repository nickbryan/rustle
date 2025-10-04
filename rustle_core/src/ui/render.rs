use crate::ui::{
    component::{Component, Element},
    values::{Color, Position, Rect},
    UiError,
};
use std::io::Error as IoError;
use taffy::{AvailableSpace, Dimension, NodeId, Size, Style, TaffyError, TaffyTree};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Canvas is an interface to the ui. It could be the terminal or web ui.
pub trait Canvas {
    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to clear the UI fails.
    fn clear(&mut self) -> Result<(), IoError>;

    /// Draw the given cells in the ui's current buffer.
    ///
    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to draw the cells fails.
    fn draw<'a, I: Iterator<Item = &'a Cell>>(&mut self, cells: I) -> Result<(), IoError>;

    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to flush the UI fails.
    fn flush(&mut self) -> Result<(), IoError>;

    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to hide the cursor fails.
    fn hide_cursor(&mut self) -> Result<(), IoError>;

    /// Position the cursor at the given row and column.
    ///
    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to position the cursor fails.
    fn position_cursor(&mut self, row: u16, col: u16) -> Result<(), IoError>;

    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to show the cursor fails.
    fn show_cursor(&mut self) -> Result<(), IoError>;

    /// Get the size of the ui.
    ///
    /// # Errors
    /// Returns an `IoError` if the underlying I/O operation to get the UI size fails.
    fn size(&self) -> Result<Rect, IoError>;
}

/// A single cell within the frame. Each cell has a position, symbol (the shown character)
/// and style information.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    position: Position,
    symbol: String,
    foreground: Color,
    background: Color,
}

impl Cell {
    /// Create a new Cell.
    #[must_use]
    pub fn new(col: u16, row: u16, symbol: &str, foreground: Color, background: Color) -> Self {
        Self {
            position: Position::new(col, row),
            symbol: symbol.into(),
            foreground,
            background,
        }
    }

    /// Returns the Position of the Cell.
    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Reset the Cell's symbol to an empty space.
    pub fn reset(&mut self) {
        self.symbol = " ".into();
    }

    /// Returns the Cell's symbol.
    #[must_use]
    pub fn symbol(&self) -> &String {
        &self.symbol
    }

    /// Returns the foreground color of this cell.
    #[must_use]
    pub fn foreground(&self) -> Color {
        self.foreground
    }

    /// Returns the background color of this cell.
    #[must_use]
    pub fn background(&self) -> Color {
        self.background
    }
}

/// Raised by the Buffer when trying to access a cell that is out of bounds.
#[derive(Error, Debug)]
#[error("trying to access index out of bounds")]
pub struct OutOfBoundsError;

/// A mapping of Cells for a given area.
///
/// All drawing within the editor will be mapped to a `Frame`. The `Frame` can then be diffed
/// with another `Frame` to detect changes that occurred within the last draw loop. This allows
/// for more efficient rendering as we only need to update changed cells and not the entire
/// screen.
pub struct Frame {
    area: Rect,
    cells: Vec<Cell>,
    cursor_position: Position,
}

impl Frame {
    #[must_use]
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Create a `Frame` with all `Cell`s having the symbol " ".
    #[must_use]
    pub fn empty(area: Rect) -> Self {
        let size = area.area();
        let mut cells = Vec::with_capacity(size);

        for row in 0..area.height {
            for col in 0..area.width {
                cells.push(Cell::new(col, row, " ", Color::Reset, Color::Reset));
            }
        }

        Self {
            cells,
            area,
            cursor_position: Position::default(),
        }
    }

    /// The current cursor position.
    #[must_use]
    pub fn cursor_position(&self) -> Position {
        self.cursor_position
    }

    /// Diff the current `Frame` with the other `Frame` to get a list of changed `Cell`s.
    fn diff<'a>(&self, other: &'a Frame) -> Vec<&'a Cell> {
        debug_assert_eq!(
            self.area, other.area,
            "Frames must be of equal size to diff"
        );
        let front_buffer = &self.cells;
        let back_buffer = &other.cells;

        let mut updates = vec![];
        for (i, (front, back)) in back_buffer.iter().zip(front_buffer.iter()).enumerate() {
            if front != back {
                updates.push(&back_buffer[i]);
            }
        }

        updates
    }

    fn index_of(&self, position: Position) -> Result<usize, OutOfBoundsError> {
        if self.area.contains(position) {
            let index = ((position.row - self.area.position.row) * self.area.width)
                + (position.col - self.area.position.col);
            Ok(index.into())
        } else {
            Err(OutOfBoundsError)
        }
    }

    /// Reset the Buffer to it's empty state.
    pub fn reset(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
    }

    /// Write a string into the `Frame`. This will overwrite any Cells currently set in the `Frame`'s
    /// given line. If the string does not fill the line it, the rest of the line will be cleared.
    ///
    /// # Errors
    ///
    /// Returns an `OutOfBoundsError` if the `position` is outside the bounds of the frame.
    pub fn write(
        &mut self,
        position: Position,
        string: &str,
        foreground: Color,
        background: Color,
    ) -> Result<(), OutOfBoundsError> {
        let str_start = self.index_of(position)?;
        let mut cursor = str_start;

        for grapheme in string[..]
            .graphemes(true)
            .take((self.area.width - position.col).into())
        {
            self.cells[cursor] = Cell::new(
                self.cells[cursor].position.col,
                self.cells[cursor].position.row,
                grapheme,
                foreground,
                background,
            );

            cursor += grapheme_width(grapheme);
        }

        for i in cursor..str_start + usize::from(self.area.width - position.col) {
            if self.cells.get(i).is_some() {
                self.cells[i].reset();
            }
        }

        Ok(())
    }

    /// Set the cursor position for the final frame render.
    pub fn set_cursor_position(&mut self, position: Position) {
        self.cursor_position = position;
    }
}

#[must_use]
pub fn grapheme_width(g: &str) -> usize {
    if g.as_bytes()[0] <= 127 {
        // Fast-path ascii.
        // Point 1: theoretically, ascii control characters should have zero
        // width, but in our case we actually want them to have width: if they
        // show up in text, we want to treat them as textual elements that can
        // be edited.  So we can get away with making all ascii single width
        // here.
        // Point 2: we're only examining the first codepoint here, which means
        // we're ignoring graphemes formed with combining characters.  However,
        // if it starts with ascii, it's going to be a single-width grapeheme
        // regardless, so, again, we can get away with that here.
        // Point 3: we're only examining the first _byte_.  But for utf8, when
        // checking for ascii range values only, that works.
        1
    } else {
        // We use max(1) here because all grapeheme clusters--even illformed
        // ones--should have at least some width so they can be edited
        // properly.
        // TODO properly handle unicode width for all codepoints
        // example of where unicode width is currently wrong: 🤦🏼‍♂️ (taken from https://hsivonen.fi/string-length/)
        UnicodeWidthStr::width(g).max(1)
    }
}

/// The area of the screen that we can draw to. The Viewport is responsible for handling
/// interactions with the `Canvas` and drawing.
pub struct Viewport<'a, C: Canvas> {
    area: Rect,
    canvas: &'a mut C,
    frames: [Frame; 2],
    current_frame_idx: usize,
}

impl<'a, C: Canvas> Viewport<'a, C> {
    /// Create a new Viewport for the provided Canvas.
    ///
    /// # Errors
    ///
    /// This function will return a `UiError::ViewportInitialization` if the `Canvas` fails to provide its size.
    pub fn new(canvas: &'a mut C) -> Result<Self, UiError> {
        let area = canvas.size().map_err(UiError::ViewportInitialization)?;

        Ok(Self {
            area,
            canvas,
            frames: [Frame::empty(area), Frame::empty(area)],
            current_frame_idx: 0,
        })
    }

    /// The area represented by the viewport.
    #[must_use]
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Draw the current `Frame` to the screen. This will call the given callback allowing the caller
    /// to define render order and cursor position. `Frame` swapping and diff are handled here to
    /// ensure that only the required screen cells are updated.
    ///
    /// # Errors
    ///
    /// This function will return a `UiError` if rendering fails. Possible error variants include:
    ///
    /// - `UiError::Render`: If there is an I/O error when interacting with the `Canvas`.
    /// - `UiError::Layout`: If the layout computation fails.
    pub fn render<S>(
        &mut self,
        state: S,
        root_component: &impl Component<S>,
    ) -> Result<(), UiError> {
        self.canvas.hide_cursor().map_err(UiError::Render)?;

        let props = root_component.select(state);

        // TODO: if props != previous props
        let element = root_component.render(props);

        let mut taffy = TaffyTree::new();
        let node = element_to_node(&mut taffy, &element)?;

        let frame = &mut self.frames[self.current_frame_idx];

        taffy.compute_layout(
            node,
            Size {
                width: AvailableSpace::Definite(f32::from(frame.area.width)),
                height: AvailableSpace::Definite(f32::from(frame.area.height)),
            },
        )?;

        render_element(&mut taffy, node, &element, frame)?;

        let next_cursor_pos = self.frames[self.current_frame_idx].cursor_position;

        let previous_frame = &self.frames[1 - self.current_frame_idx];
        let changes = previous_frame.diff(&self.frames[self.current_frame_idx]);

        self.canvas
            .draw(changes.into_iter())
            .map_err(UiError::Render)?;

        self.canvas
            .position_cursor(next_cursor_pos.row, next_cursor_pos.col)
            .map_err(UiError::Render)?;

        self.canvas.show_cursor().map_err(UiError::Render)?;

        self.swap_buffers();

        self.canvas.flush().map_err(UiError::Render)
    }

    fn swap_buffers(&mut self) {
        self.frames[1 - self.current_frame_idx].reset();
        self.current_frame_idx = 1 - self.current_frame_idx;
    }
}

impl<G: Canvas> Drop for Viewport<'_, G> {
    /// When the Viewport goes out of scope (application has ended) we want to ensure that the
    /// screen is cleared and flushed to leave the user with a clean terminal.
    fn drop(&mut self) {
        // In a drop implementation, we should not panic. If the canvas operations fail,
        // there is little we can do to recover, so we ignore the errors.
        let _ = self.canvas.clear();
        let _ = self.canvas.flush();
    }
}

fn element_to_node(taffy: &mut TaffyTree, element: &Element) -> Result<NodeId, TaffyError> {
    match element {
        Element::Span(span) => {
            // The `cast_precision_loss` lint is allowed here because the precision loss from
            // converting the text length to `f32` is not critical for the layout calculation
            // in this context.
            #[allow(clippy::cast_precision_loss)]
            let style = Style {
                size: Size {
                    width: Dimension::length(span.text.len() as f32),
                    height: Dimension::length(1.0),
                },
                ..Default::default()
            };

            taffy.new_leaf(style)
        }
        Element::Container(container) => {
            let children = container
                .children
                .iter()
                .map(|child| element_to_node(taffy, child))
                .collect::<Result<Vec<NodeId>, _>>()?;

            taffy.new_with_children(container.layout.clone(), &children)
        }
    }
}

fn render_element(
    taffy: &mut TaffyTree,
    node_id: NodeId,
    element: &Element,
    frame: &mut Frame,
) -> Result<(), UiError> {
    let layout = taffy.layout(node_id)?;
    // The `cast_possible_truncation` and `cast_sign_loss` lints are allowed here because
    // the layout coordinates are expected to be within the valid range of `u16` and non-negative,
    // so the truncation and sign loss are not a concern.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let position = Position {
        col: layout.location.x as u16,
        row: layout.location.y as u16,
    };

    match element {
        Element::Span(span) => {
            frame
                .write(position, &span.text, span.color, span.background)
                .map_err(|e| UiError::Render(IoError::other(e)))?;
        }
        Element::Container(container) => {
            let children = taffy.children(node_id).unwrap(); // Should not fail if layout is valid

            for (i, child) in container.children.iter().enumerate() {
                let child_node = children[i];
                render_element(taffy, child_node, child, frame)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_data_is_stored_correctly() {
        let cell = Cell::new(10, 10, "a", Color::Red, Color::White);

        assert_eq!(cell.position().col, 10);
        assert_eq!(cell.position().row, 10);
        assert_eq!(cell.symbol(), "a");
        assert_eq!(cell.foreground(), Color::Red);
        assert_eq!(cell.background(), Color::White);
    }

    #[test]
    fn cell_has_empty_symbol_when_reset() {
        let mut cell = Cell::new(10, 10, "a", Color::Red, Color::White);

        cell.reset();

        assert_eq!(cell.position().col, 10);
        assert_eq!(cell.position().row, 10);
        assert_eq!(cell.symbol(), " ");
        assert_eq!(cell.foreground(), Color::Red);
        assert_eq!(cell.background(), Color::White);
    }

    #[test]
    fn frame_has_no_diff_when_both_are_empty() {
        let frame = Frame::empty(Rect::new(10, 10));
        let other = Frame::empty(Rect::new(10, 10));
        let empty_diff: Vec<&Cell> = vec![];
        assert_eq!(empty_diff, frame.diff(&other));
    }

    #[test]
    fn frame_produces_diff_when_written_to() {
        let mut dirty = Frame::empty(Rect::new(10, 10));
        let clean = Frame::empty(Rect::new(10, 10));

        let _ = dirty.write(Position::new(0, 0), "hello", Color::DarkGray, Color::White);
        assert_eq!(
            vec![
                &Cell {
                    position: Position { col: 0, row: 0 },
                    symbol: "h".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 1, row: 0 },
                    symbol: "e".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 2, row: 0 },
                    symbol: "l".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 3, row: 0 },
                    symbol: "l".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 4, row: 0 },
                    symbol: "o".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
            ],
            clean.diff(&dirty),
        );
    }

    #[test]
    fn frame_resets_cells_to_empty() {
        let mut dirty = Frame::empty(Rect::new(10, 10));
        let clean = Frame::empty(Rect::new(10, 10));

        let _ = dirty.write(Position::new(0, 0), "hello", Color::DarkGray, Color::White);
        dirty.reset();

        assert_eq!(
            vec![
                &Cell {
                    position: Position { col: 0, row: 0 },
                    symbol: " ".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 1, row: 0 },
                    symbol: " ".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 2, row: 0 },
                    symbol: " ".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 3, row: 0 },
                    symbol: " ".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
                &Cell {
                    position: Position { col: 4, row: 0 },
                    symbol: " ".to_string(),
                    foreground: Color::DarkGray,
                    background: Color::White,
                },
            ],
            clean.diff(&dirty),
        );
    }

    #[test]
    fn frame_persists_cursor_position() {
        let mut frame = Frame::empty(Rect::default());
        assert_eq!(Position::default(), frame.cursor_position());
        frame.set_cursor_position(Position::new(20, 20));
        assert_eq!(Position::new(20, 20), frame.cursor_position());
    }
}
