//! # Rustle Core
//!
//! This crate provides the core functionality for the Rustle text editor, including
//! an actor-based state management system inspired by Redux.

mod state;
mod editor;
pub mod ui;
mod input;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};