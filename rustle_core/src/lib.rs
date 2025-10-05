//! # Rustle Core
//!
//! This crate provides the core functionality for the Rustle text editor, including
//! an actor-based state management system inspired by Redux.

#![warn(clippy::all, clippy::pedantic)]

mod editor;
mod error;
mod input;
mod ui;

pub use editor::Editor;
pub use error::Error;
pub use input::{Event, EventStream, Key};
pub use ui::{Canvas, Cell, Color, Position, Rect};
