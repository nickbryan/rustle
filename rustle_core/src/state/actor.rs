use async_trait::async_trait;
use crate::{
    state::{
        mailbox::Address,
        mailbox::Mailbox,
        reducer::Reducer,
        mailbox::Deliver,
        message::Dispatch
    }
};

pub struct Actor<R, S, A>
where
    S: Send,
    R: Send,
{
    mailbox: Mailbox<R, S, A>,
    root_reducer: R,
    state: Option<S>,
}

impl <R, S, A> Actor<R, S, A>
where
    S: Send,
    R: Reducer<S, A> + Send,
{
    pub fn new(root_reducer: R, state: S) -> Self {
        Self {
            mailbox: Mailbox::new(),
            root_reducer,
            state: Some(state),
        }
    }

    pub fn mailbox(&self) -> Address<R, S, A> {
        self.mailbox.address()
    }

    pub async fn act(&mut self) {
        while let Some(message) = self.mailbox.recv().await {
            message.dispatch(self).await;
        }
    }
}

#[async_trait]
impl <R, S, A> Deliver<Dispatch<A>> for Actor<R, S, A>
where
    R: Reducer<S, A> + Send,
    S: Send,
    A: Send,
{
    async fn deliver(&mut self, message: Dispatch<A>) {
        let action = message.into_action();

        let old_state = self.state.take().unwrap();
        let new_state = self.root_reducer.reduce(old_state, action);

        self.state = Some(new_state);
    }
}