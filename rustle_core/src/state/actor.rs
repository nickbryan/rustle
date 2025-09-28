use async_trait::async_trait;
use crate::{ 
    state::{
        mailbox::Address,
        mailbox::Mailbox,
        reducer::Reducer,
        mailbox::Deliver,
        message::Dispatch,
        message::Select
    }
};

use tokio::sync::watch;
use crate::state::selector::Selector;

/// The `Actor` is the core of the state management system, acting as the central
/// processing unit. It runs in a separate, dedicated task and is responsible for
/// managing the state, processing incoming messages, and notifying subscribers of
/// state changes.
///
/// The actor's primary responsibilities include:
///
/// * **State Management**: Holding the single, authoritative state of the application.
/// * **Message Processing**: Receiving and handling messages from its mailbox, such as
///   `Dispatch` and `Select` messages.
/// * **State Mutation**: Applying the root reducer to the current state and an action to
///   produce a new state.
/// * **Notification**: Broadcasting state changes to all subscribers.
///
/// This design ensures that all state mutations are handled sequentially and safely,
/// preventing race conditions and ensuring data consistency in a concurrent environment.
pub struct Actor<R, S, A>
where
    S: Send + Clone + Sync,
    R: Send,
{
    mailbox: Mailbox<R, S, A>,
    root_reducer: R,
    state: Option<S>,
    notifier: watch::Sender<S>,
}

impl <R, S, A> Actor<R, S, A>
where
    S: Send + Clone + Sync,
    R: Reducer<S, A> + Send,
{
    /// Creates a new actor with a given reducer and initial state.
    pub fn new(root_reducer: R, state: S) -> Self {
        let (notifier, _) = watch::channel(state.clone());
        Self {
            mailbox: Mailbox::new(),
            root_reducer,
            state: Some(state),
            notifier,
        }
    }

    /// Returns a sender for the notifier.
    /// The notifier is used to broadcast state changes to subscribers.
    pub fn notifier(&self) -> watch::Sender<S> {
        self.notifier.clone()
    }

    /// Returns the address of the actor's mailbox.
    /// The address is used to send messages to the actor.
    pub fn mailbox(&self) -> Address<R, S, A> {
        self.mailbox.address()
    }

    /// Starts the actor's event loop.
    /// The actor will continuously receive and process messages from its mailbox.
    pub async fn act(&mut self) {
        while let Some(assignment) = self.mailbox.recv().await {
            assignment.assign(self).await;

        }
    }
}

#[async_trait]
impl <R, S, A> Deliver<Dispatch<A>> for Actor<R, S, A>
where
    R: Reducer<S, A> + Send,
    S: Send + Clone + Sync,
    A: Send,
{
    /// Handles a `Dispatch` message.
    /// This will run the reducer and update the state.
    async fn deliver(&mut self, message: Dispatch<A>) {
        let action = message.into_action();

        let old_state = self.state.take().expect("State should always be Some");
        let new_state = self.root_reducer.reduce(old_state, action);

        self.state = Some(new_state.clone());
        let _ = self.notifier.send(new_state); // TODO: handle result and error.
    }
}

#[async_trait]
impl<R, S, A, Sel, Result> Deliver<Select<S, Sel>> for Actor<R, S, A>
where
    R: Reducer<S, A> + Send,
    S: Send + Clone + Sync,
    A: Send,
    Sel: Selector<S, Result = Result> + Send + 'static,
    Result: Send,
{
    /// Handles a `Select` message.
    /// This will run a selector on the current state and return the result.
    async fn deliver(&mut self, message: Select<S, Sel>) -> Result {
        let state = self.state.as_ref().expect("State should always be Some");
        let selector = message.into_selector();
        selector.select(state)
    }
}