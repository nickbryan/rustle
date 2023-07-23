#![warn(clippy::all, clippy::pedantic)]

mod communication;
mod component;
mod editor;
mod input;
mod render;

pub mod ui;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};
pub use render::{Canvas, Cell};
