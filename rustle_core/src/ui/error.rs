use std::io;

use taffy::TaffyError;
use thiserror::Error;

/// Represents errors originating from the UI, such as rendering or viewport setup.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initialize the UI viewport")]
    ViewportInitialization(#[source] io::Error),

    #[error("Failed to render the UI")]
    Render(#[source] io::Error),

    #[error("Layout computation failed")]
    Layout(#[from] TaffyError),
}
