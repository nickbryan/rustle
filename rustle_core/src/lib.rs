#![warn(clippy::all, clippy::pedantic)]
mod component;
mod editor;
mod input;
mod mode;
mod render;

mod graphemes;
pub mod ui;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};
pub use render::{Canvas, Cell};

use mode::Mode;
