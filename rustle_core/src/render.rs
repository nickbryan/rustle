use crate::ui::{Color, Position, Rect};
use anyhow::Result;
use std::io::Error as IoError;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

/// Canvas is an interface to the ui. It could be the terminal or web ui.
pub trait Canvas {
    /// Clear the ui.
    fn clear(&mut self) -> Result<(), IoError>;

    /// Draw the given cells in the ui's current buffer.
    fn draw<'a, I: Iterator<Item = &'a Cell>>(&mut self, cells: I) -> Result<(), IoError>;

    /// Flush the ui's current buffer.
    fn flush(&mut self) -> Result<(), IoError>;

    /// Hide the cursor.
    fn hide_cursor(&mut self) -> Result<(), IoError>;

    /// Position the cursor at the given row and column.
    fn position_cursor(&mut self, row: usize, col: usize) -> Result<(), IoError>;

    /// Show the cursor.
    fn show_cursor(&mut self) -> Result<(), IoError>;

    /// Get the size of the ui.
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
    pub fn new(col: usize, row: usize, symbol: &str, foreground: Color, background: Color) -> Self {
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
    /// Create a `Frame` with all `Cell`s having the symbol " ".
    pub fn empty(area: Rect) -> Self {
        Self::filled(area, " ")
    }

    /// Create a Frame` with all `Cell`s set to the given symbol.
    pub fn filled(area: Rect, symbol: &str) -> Self {
        let size = area.area();
        let mut cells = Vec::with_capacity(size);

        for row in 0..area.height {
            for col in 0..area.width {
                cells.push(Cell::new(col, row, symbol, Color::Reset, Color::Reset));
            }
        }

        Self {
            cells,
            area,
            cursor_position: Position::default(),
        }
    }

    /// The current cursor position.
    pub fn cursor_position(&self) -> Position {
        self.cursor_position
    }

    /// Diff the current `Frame` with the other `Frame` to get a list of changed `Cell`s.
    fn diff<'a>(&self, other: &'a Frame) -> Vec<&'a Cell> {
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

    fn index_of(&self, position: &Position) -> Result<usize, OutOfBoundsError> {
        if self.area.contains(position) {
            Ok((position.row - self.area.position.row) * self.area.width
                + (position.col - self.area.position.col))
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

    /// Write a line into the `Frame`. This will overwrite any Cells currently set in the `Frame`'s
    /// given line. If the string does not fill the line it, the rest of the line will be cleared.
    pub fn write_line(
        &mut self,
        row_number: usize,
        string: &str,
        foreground: Color,
        background: Color,
    ) {
        let index = self.index_of(&Position::new(0, row_number)).unwrap();

        for (i, grapheme) in string[..].graphemes(true).enumerate() {
            // TODO: do we want to cap the line length here? If the line is longer than the width do we truncate?

            let cell_idx = index + i;
            self.cells[cell_idx] = Cell::new(
                self.cells[cell_idx].position.col,
                self.cells[cell_idx].position.row,
                grapheme,
                foreground,
                background,
            );
        }

        for i in index + string[..].graphemes(true).count()..index + self.area.width {
            self.cells[i].reset();
        }
    }

    /// Set the cursor position for the final frame render.
    pub fn set_cursor_position(&mut self, position: Position) {
        self.cursor_position = position;
    }
}

/// `View` can be implemented on any `Component` to allow it to be drawn to the `Viewport`.
pub trait View {
    fn render_to(&self, frame: &mut Frame);
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
    pub fn new(canvas: &'a mut C) -> Result<Self> {
        use anyhow::Context;

        let area = canvas.size().context("unable to set Viewport area")?;

        Ok(Self {
            area,
            canvas,
            frames: [Frame::empty(area), Frame::empty(area)],
            current_frame_idx: 0,
        })
    }

    /// The area represented by the viewport.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Draw the current `Frame` to the screen. This will call the given callback allowing the caller
    /// to define render order and cursor position. `Frame` swapping and diff is handled here to
    /// ensure that only the required screen cells are updated.
    pub fn render<V: View>(&mut self, view: &V) -> Result<()> {
        use anyhow::Context;

        self.canvas
            .hide_cursor()
            .context("unable to hide cursor pre draw")?;

        view.render_to(&mut self.frames[self.current_frame_idx]);

        let next_cursor_pos = self.frames[self.current_frame_idx].cursor_position;

        let previous_frame = &self.frames[1 - self.current_frame_idx];
        let changes = previous_frame.diff(&self.frames[self.current_frame_idx]);

        self.canvas
            .draw(changes.into_iter())
            .context("unable to draw buffer diff")?;

        self.canvas
            .position_cursor(next_cursor_pos.row, next_cursor_pos.col)
            .context("unable to set cursor position for next frame render")?;

        self.canvas
            .show_cursor()
            .context("unable to show cursor post draw")?;

        self.swap_buffers();

        self.canvas.flush().context("unable to flush canvas")
    }

    fn swap_buffers(&mut self) {
        self.frames[1 - self.current_frame_idx].reset();
        self.current_frame_idx = 1 - self.current_frame_idx;
    }
}

impl<'a, G: Canvas> Drop for Viewport<'a, G> {
    /// When the Viewport goes out of scope (application has ended) we want to ensure that the
    /// screen is cleared and flushed to leave the user with a clean terminal.
    fn drop(&mut self) {
        self.canvas.clear().unwrap();
        self.canvas.flush().unwrap();
    }
}
