//! # Rustle Core
//!
//! This crate provides the core functionality for the Rustle text editor, including
//! an actor-based state management system inspired by Redux.

mod state;
mod editor;
mod ui;

pub use editor::Editor;