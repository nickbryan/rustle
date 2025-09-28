
use async_trait::async_trait;
use tokio::{ 
    sync::{ 
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
    }
};
use crate::state::actor::Actor;
use crate::state::error::StateError;

/// A message that can be sent to an actor.
/// This is a trait that all messages must implement.
/// The `Response` associated type defines the type of the response that the actor will send back.
pub trait Message: Send {
    type Response: Send;
}

/// An envelope is a wrapper around a message that includes a sender for the response.
struct Envelope<M: Message> {
    message: M,
    sender: Sender<M::Response>
}

impl<M: Message> Envelope<M> {
    fn new(message: M, sender: Sender<M::Response>) -> Self {
        Self { message, sender }
    }
}

/// `Deliver` is a trait implemented by handlers to instruct them what to do with a message.
#[async_trait]
pub trait Deliver<M: Message> {
    async fn deliver(&mut self, message: M) -> M::Response;
}

/// Assign is a trait implemented for Envelope to assign itself to an Actor.
#[async_trait]
pub trait Assign<A> {
    async fn assign(self: Box<Self>, courier: &mut A);
}

#[async_trait]
impl<A, M> Assign<A> for Envelope<M>
where
    A: Deliver<M> + Send,
    M: Message + Send,
    Self: Send,
{
    /// Assigns the message to the handler. The handler will send the response back to the sender.
    async fn assign(self: Box<Self>, courier: &mut A) {
        let reply = courier.deliver(self.message).await;
        let _ = self.sender.send(reply);  // TODO: handle result and error.
    }
}

/// An `Assignment` allows a message to be assigned to a handler.
type Assignment<R, S, A> = Box<dyn Assign<Actor<R, S, A>> + Send>;

/// A `Mailbox` is a message queue that holds incoming messages for an actor.
/// It ensures that messages are processed sequentially, preserving the order in which
/// they were sent.
///
/// The `Mailbox` is a key component of the actor model, providing a safe and reliable
/// communication channel between the actor and the rest of the application.
pub struct Mailbox<R: Send, S: Send + Clone + Sync, A> {
    rx: UnboundedReceiver<Assignment<R, S, A>>,
    tx: UnboundedSender<Assignment<R, S, A>>,
}

impl<R: Send, S: Send + Clone + Sync, A> Mailbox<R, S, A> {
    /// Creates and returns a new Mailbox.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { rx, tx }
    }

    /// Returns an address for the mailbox.
    /// This allows messages to be sent to the mailbox.
    pub fn address(&self) -> Address<R, S, A> {
        Address::new(self.tx.clone())
    }

    /// Receives a message from the mailbox.
    /// Returns `None` if the mailbox is empty.
    pub async fn recv(&mut self) -> Option<Assignment<R, S, A>> {
        self.rx.recv().await
    }
}

/// An `Address` is a lightweight handle to an actor's mailbox. It allows other parts
/// of the application to send messages to the actor without having direct access to
/// the actor itself.
///
/// The `Address` decouples the message sender from the receiver.
pub struct Address<R: Send, S: Send + Clone + Sync, A> {
    tx: UnboundedSender<Assignment<R, S, A>>,
}

impl<R: Send, S: Send + Clone + Sync, A> Address<R, S, A> {
    /// Creates and returns a new address.
    fn new(tx: UnboundedSender<Assignment<R, S, A>>) -> Self {
        Self { tx }
    }

    /// Send a message to the `Mailbox`.
    pub async fn send<M: Message + 'static>(&self, message: M) -> Result<M::Response, StateError>
    where
        Actor<R, S, A>: Deliver<M>
    {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Box::new(Envelope::new(message, tx))).map_err(|_| StateError::ActorTerminated)?;
        rx.await.map_err(|_| StateError::ActorTerminated)
    }
}