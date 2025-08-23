use crate::state::mailbox::Message;

pub struct Dispatch<A: Send> {
    action: A,
}

impl<A: Send> Dispatch<A> {
    pub fn new(action: A) -> Self {
        Self { action }
    }

    pub fn into_action(self) -> A {
        self.action
    }
}

impl<A: Send> Message for Dispatch<A> {
    type Reply = ();
}