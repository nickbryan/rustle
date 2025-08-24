use async_trait::async_trait;
use tokio::{ 
    sync::{ 
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
    }
};
use crate::state::actor::Actor;

pub trait Message: Send {
    type Response: Send;
}

struct Envelope<M: Message> {
    message: M,
    sender: Sender<M::Response>
}

impl<M: Message> Envelope<M> {
    fn new(message: M, sender: Sender<M::Response>) -> Self {
        Self { message, sender }
    }
}

#[async_trait]
pub trait Handle<M: Message> {
    async fn handle(&mut self, message: M) -> M::Response;
}

#[async_trait]
pub trait Assign<C> {
    async fn assign(self: Box<Self>, handler: &mut C);
}

#[async_trait]
impl<H, M> Assign<H> for Envelope<M>
where
    H: Handle<M> + Send,
    M: Message + Send,
    Self: Send,
{
    async fn assign(self: Box<Self>, handler: &mut H) {
        let reply = handler.handle(self.message).await;
        let _ = self.sender.send(reply); // TODO: address if this error needs handling.
    }
}

type Assignment<R, S, A> = Box<dyn Assign<Actor<R, S, A>> + Send>;

pub struct Mailbox<R: Send, S: Send + Clone + Sync, A> {
    rx: UnboundedReceiver<Assignment<R, S, A>>,
    tx: UnboundedSender<Assignment<R, S, A>>,
}

impl<R: Send, S: Send + Clone + Sync, A> Mailbox<R, S, A> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    pub fn address(&self) -> Address<R, S, A> {
        Address::new(self.tx.clone())
    }

    pub async fn recv(&mut self) -> Option<Assignment<R, S, A>> {
        self.rx.recv().await
    }
}

pub struct Address<R: Send, S: Send + Clone + Sync, A> {
    tx: UnboundedSender<Assignment<R, S, A>>,
}

impl<R: Send, S: Send + Clone + Sync, A> Address<R, S, A> {
    fn new(tx: UnboundedSender<Assignment<R, S, A>>) -> Self {
        Self { tx }
    }

    pub async fn send<M: Message + 'static>(&self, message: M) -> M::Response
    where
        Actor<R, S, A>: Handle<M>
    {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Box::new(Envelope::new(message, tx))); // TODO: address if this error needs handling.
        rx.await.unwrap() // TODO: address if this error needs handling.
    }
}