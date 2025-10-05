use rustle_state::StateError;
use thiserror::Error;

use crate::ui::UiError;

/// The primary, top-level error type for the `rustle_core` crate.
#[derive(Error, Debug)]
pub enum CoreError {
    /// An error originating from the state management module.
    #[error(transparent)]
    State(#[from] StateError),

    /// An error originating from the UI module.
    #[error(transparent)]
    Ui(#[from] UiError),

    /// An error related to reading input events.
    #[error("Failed to read input event: {0}")]
    Input(String),
}
