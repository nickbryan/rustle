#![warn(clippy::all, clippy::pedantic)]

pub use editor::Editor;
pub use input::{Event, EventStream, Key};
use mode::Mode;
pub use render::{Canvas, Cell};

mod component;
mod editor;
mod input;
mod mode;
mod render;

mod graphemes;
pub mod ui;

