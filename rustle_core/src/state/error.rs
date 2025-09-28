use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Actor is no longer running.")]
    ActorTerminated,
}