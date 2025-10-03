//! # Rustle Core
//!
//! This crate provides the core functionality for the Rustle text editor, including
//! an actor-based state management system inspired by Redux.

#![warn(clippy::all, clippy::pedantic)]

pub mod editor;
mod input;
mod state;
pub mod ui;

pub mod error;
pub use error::CoreError;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};
