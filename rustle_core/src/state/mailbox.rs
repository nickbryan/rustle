use async_trait::async_trait;
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
    }
};
use crate::state::actor::Actor;

pub trait Message: Send {
    type Reply: Send;
}

struct Envelope<M: Message> {
    message: M,
    sender: Sender<M::Reply>
}

impl<M: Message> Envelope<M> {
    fn new(message: M, sender: Sender<M::Reply>) -> Self {
        Self { message, sender }
    }
}

#[async_trait]
pub trait Deliver<M: Message> {
    async fn deliver(&mut self, message: M) -> M::Reply;
}

#[async_trait]
pub trait Dispatch<C> {
    async fn dispatch(self: Box<Self>, courier: &mut C);
}

#[async_trait]
impl<C, M> Dispatch<C> for Envelope<M>
where
    C: Deliver<M> + Send,
    M: Message + Send,
    Self: Send,
{
    async fn dispatch(self: Box<Self>, courier: &mut C) {
        let reply = courier.deliver(self.message).await;
        let _ = self.sender.send(reply); // TODO: address if this error needs handling.
    }
}

type MessageBox<R, S, A> = Box<dyn Dispatch<Actor<R, S, A>> + Send>;

pub struct Mailbox<R: Send, S: Send + Clone + Sync, A> {
    rx: UnboundedReceiver<MessageBox<R, S, A>>,
    tx: UnboundedSender<MessageBox<R, S, A>>,
}

impl<R: Send, S: Send + Clone + Sync, A> Mailbox<R, S, A> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    pub fn address(&self) -> Address<R, S, A> {
        Address::new(self.tx.clone())
    }

    pub async fn recv(&mut self) -> Option<MessageBox<R, S, A>> {
        self.rx.recv().await
    }
}

pub struct Address<R: Send, S: Send + Clone + Sync, A> {
    tx: UnboundedSender<MessageBox<R, S, A>>,
}

impl<R: Send, S: Send + Clone + Sync, A> Address<R, S, A> {
    fn new(tx: UnboundedSender<MessageBox<R, S, A>>) -> Self {
        Self { tx }
    }

    pub async fn send<M: Message + 'static>(&self, message: M) -> M::Reply
    where
        Actor<R, S, A>: Deliver<M>
    {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Box::new(Envelope::new(message, tx))); // TODO: address if this error needs handling.
        rx.await.unwrap() // TODO: address if this error needs handling.
    }
}