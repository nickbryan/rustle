use tokio::sync::watch;

use crate::{
    actor::Actor,
    mailbox::Address,
    message::{Dispatch, Select},
    reducer::Reducer,
    selector::Selector,
};

/// A trait for spawning asynchronous tasks.
/// This allows the store to be used with different runtimes, such as Tokio or async-std.
pub trait Runtime {
    /// Spawns a new asynchronous task.
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);
}

/// The `Store` is the primary interface for interacting with the application state.
/// It provides a simple and consistent way to manage state in a concurrent environment.
///
/// The `Store` is responsible for:
///
/// * Dispatching actions to the reducer to trigger state changes.
/// * Selecting data from the state using selectors.
/// * Notifying subscribers of state changes.
///
/// The `Store` is generic over the reducer, state, and action types, making it highly
/// flexible and adaptable to different use cases.
pub struct Store<R, S, A>
where
    R: Send,
    S: Send + Clone + Sync,
{
    mailbox: Address<R, S, A>,
    subscription: watch::Receiver<S>,
}

impl<R, S, A> Store<R, S, A>
where
    S: Send + 'static + Clone + Sync,
    A: Send + 'static,
    R: Reducer<S, A> + Send + 'static,
{
    /// Creates a new store with a given initial state and root reducer.
    pub fn new<RT: Runtime>(root_reducer: R, state: S, runtime: &RT) -> Self {
        let mut actor = Actor::new(root_reducer, state);
        let mailbox = actor.mailbox();
        let subscription = actor.notifier().subscribe();

        runtime.spawn(async move { actor.act().await });

        Self {
            mailbox,
            subscription,
        }
    }

    /// Dispatches an action to the store.
    /// The action will be processed by the reducer, which will update the state.
    /// This is the only way to trigger a state change.
    pub async fn dispatch(&self, action: A) -> Result<(), super::error::StateError> {
        self.mailbox.send(Dispatch::new(action)).await
    }

    /// Selects a value from the state using a selector.
    /// Selectors are used to derive data from the state.
    pub async fn select<Sel, Res>(&self, selector: Sel) -> Result<Res, super::error::StateError>
    where
        Sel: Selector<S, Result = Res> + Send + 'static,
        Res: Send + 'static,
    {
        self.mailbox.send(Select::new(selector)).await
    }

    /// Subscribes to state changes.
    /// Returns a `watch::Receiver` that can be used to receive notifications when the state changes.
    pub fn subscribe(&self) -> watch::Receiver<S> {
        self.subscription.clone()
    }
}
