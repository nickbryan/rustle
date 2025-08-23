use crate::{
    state::{
        actor::Actor,
        mailbox::Address,
        message::Dispatch
    }
};
use crate::state::reducer::Reducer;

/// A store is a container for the state.
pub struct Store<R: Send, S: Send, A> {
    mailbox: Address<R, S, A>,
}

impl<R, S, A> Store<R, S, A>
where
    S: Send + 'static,
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

        let _ = tokio::spawn(async move {
            actor.act().await;
        });

        Self {
            mailbox,
        }
    }

    pub async fn dispatch(&self, action: A) {
        self.mailbox.send(Dispatch::new(action)).await;
    }
}