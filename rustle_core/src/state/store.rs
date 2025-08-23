use crate::{
    state::{
        actor::Actor,
        mailbox::Address,
        message::Dispatch
    }
};
use crate::state::reducer::Reducer;
use tokio::sync::watch;

/// A store is a container for the state.
pub struct Store<R: Send, S: Send + Clone + Sync, A> {
    mailbox: Address<R, S, A>,
    receiver: watch::Receiver<S>,
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
        let receiver = actor.notifier().subscribe();

        let _ = tokio::spawn(async move {
            actor.act().await;
        });

        Self {
            mailbox,
            receiver,
        }
    }

    pub async fn dispatch(&self, action: A) {
        self.mailbox.send(Dispatch::new(action)).await;
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&S) + Send + Sync + 'static,
    {
        let mut receiver = self.receiver.clone();

        tokio::spawn(async move {
            callback(&receiver.borrow());

            while receiver.changed().await.is_ok() {
                callback(&receiver.borrow());
            }
        });
    }
}
