use crate::{ 
    state::{ 
        actor::Actor,
        mailbox::Address,
        message::{ 
            Dispatch,
            Select
        },
        reducer::Reducer,
        selector::Selector
    }
};
use tokio::sync::watch;

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
pub struct Store<R: Send, S: Send + Clone + Sync, A> {
    mailbox: Address<R, S, A>,
    subscription: watch::Receiver<S>,
}

impl<R, S, A> Store<R, S, A>
where
    S: Send + 'static + Clone + Sync,
    A: Send + 'static,
    R: Reducer<S, A> + Send + 'static,
{
    /// Creates a new store with a default initial state.
    pub fn new(root_reducer: R) -> Self
    where
        S: Default
    {
        Self::new_with_state(root_reducer, Default::default())
    }

    /// Creates a new store with a given initial state.
    pub fn new_with_state(root_reducer: R, state: S) -> Self {
        let mut actor = Actor::new(root_reducer, state);
        let mailbox = actor.mailbox();
        let subscription = actor.notifier().subscribe();

        let _ = tokio::spawn(async move {
            actor.act().await;
        });

        Self {
            mailbox,
            subscription,
        }
    }

    /// Dispatches an action to the store.
    /// The action will be processed by the reducer, which will update the state.
    /// This is the only way to trigger a state change.
    pub async fn dispatch(&self, action: A) {
        self.mailbox.send(Dispatch::new(action)).await;
    }

    /// Selects a value from the state using a selector.
    /// Selectors are used to derive data from the state.
    pub async fn select<Sel: Selector<S, Result = Res>, Res>(&self, selector: Sel) -> Res
    where
        Sel: Selector<S, Result = Res> + Send + 'static,
        Res: Send + 'static,
    {
        self.mailbox.send(Select::new(selector)).await
    }

    /// Subscribes to state changes.
    /// The provided callback will be called whenever the state changes.
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&S) + Send + Sync + 'static,
    {
        let mut state = self.subscription.clone();

        tokio::spawn(async move {
            callback(&state.borrow());

            while state.changed().await.is_ok() {
                callback(&state.borrow());
            }
        });
    }
}