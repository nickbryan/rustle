use thiserror::Error;

/// Represents errors that can occur within the state management system.
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Actor is no longer running.")]
    ActorTerminated,
}
