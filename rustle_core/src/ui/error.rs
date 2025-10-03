use taffy::TaffyError;
use thiserror::Error;

/// Represents errors originating from the UI, such as rendering or viewport setup.
#[derive(Error, Debug)]
pub enum UiError {
    #[error("Failed to initialize the UI viewport")]
    ViewportInitialization(#[source] std::io::Error),

    #[error("Failed to render the UI")]
    Render(#[source] std::io::Error),

    #[error("Layout computation failed")]
    Layout(#[from] TaffyError),
}
