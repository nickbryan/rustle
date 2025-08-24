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

/// A store is a container for the state.
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
    pub fn new(root_reducer: R) -> Self
    where
        S: Default
    {
        Self::new_with_state(root_reducer, Default::default())
    }

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

    pub async fn dispatch(&self, action: A) {
        self.mailbox.send(Dispatch::new(action)).await;
    }

    pub async fn select<Sel: Selector<S, Result = Res>, Res>(&self, selector: Sel) -> Res
    where
        Sel: Selector<S, Result = Res> + Send + 'static,
        Res: Send + 'static,
    {
        self.mailbox.send(Select::new(selector)).await
    }

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