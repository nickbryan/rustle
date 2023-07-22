#![warn(clippy::all, clippy::pedantic)]

mod editor;
mod input;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};
