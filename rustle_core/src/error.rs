use rustle_state::StateError;
use thiserror::Error as ThisError;

use crate::ui::Error as UIError;

/// The primary, top-level error type for the `rustle_core` crate.
#[derive(ThisError, Debug)]
pub enum Error {
    /// An error originating from the state management module.
    #[error(transparent)]
    State(#[from] StateError),

    /// An error originating from the UI module.
    #[error(transparent)]
    Ui(#[from] UIError),

    /// An error related to reading input events.
    #[error("Failed to read input event: {0}")]
    Input(String),
}
