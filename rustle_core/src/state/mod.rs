//! # Actor-based State Management
//!
//! This module contains the core components of a Redux-like state management system,
//! built upon the actor pattern. This pattern ensures that all state mutations are
//! handled sequentially and safely in a concurrent environment.
//!
//! ## Core Concepts
//!
//! * **Actor**: The central processing unit of the state management system. It runs in a
//!   separate task and is responsible for receiving messages, processing them, and updating
//!   the state.
//!
//! * **Store**: The primary interface for interacting with the state. It provides methods
//!   for dispatching actions, selecting data from the state, and subscribing to state
//!   changes.
//!
//! * **Reducer**: A pure function that takes the current state and an action, and returns
//!   the new state. Reducers are the only place where state can be changed.
//!
//! * **Action**: A message that represents an intent to change the state. Actions are
//!   dispatched to the store, which then forwards them to the actor.
//!
//! * **Selector**: A pure function that takes the state and returns some derived data.
//!   Selectors are used to encapsulate the logic for deriving data from the state.
//!
//! * **Mailbox**: A message queue for the actor. It allows messages to be sent to the
//!   actor from other parts of the application.
//!
//! ## Flow
//!
//! 1. An action is dispatched to the store.
//! 2. The store sends a `Dispatch` message to the actor's mailbox.
//! 3. The actor receives the message and calls the root reducer with the current state
//!    and the action.
//! 4. The reducer returns a new state, which the actor then stores.
//! 5. The actor notifies all subscribers of the state change.
//!
//! This architecture ensures that the state is always predictable and that all state
//! changes are explicit and traceable.

mod actor;
mod mailbox;
mod message;
mod reducer;
mod selector;
mod store;

pub use store::Store;

pub mod error;
pub use error::StateError;

