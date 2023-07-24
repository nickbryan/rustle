#![warn(clippy::all, clippy::pedantic)]

mod communication;
mod component;
mod document;
mod editor;
mod input;
mod mode;
mod render;
mod row;

pub mod ui;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};
pub use render::{Canvas, Cell};

use mode::Mode;
use row::Row;
