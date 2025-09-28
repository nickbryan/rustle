//! # Rustle Core
//!
//! This crate provides the core functionality for the Rustle text editor, including
//! an actor-based state management system inspired by Redux.

#![warn(clippy::all, clippy::pedantic)]

mod state;
pub mod editor;
pub mod ui;
mod input;

pub mod error;
pub use error::CoreError;

pub use editor::Editor;
pub use input::{Event, EventStream, Key};