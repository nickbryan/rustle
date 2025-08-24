use std::marker::PhantomData;
use crate::{ 
    state::{ 
        mailbox::Message,
        selector::Selector
    }
};

/// A `Dispatch` message is a wrapper around an action that is sent to the store to
/// trigger a state change. It is the sole mechanism for initiating state mutations.
///
/// When a `Dispatch` message is sent, the store forwards it to the actor, which then
/// invokes the root reducer to update the state.
pub struct Dispatch<A: Send> {
    action: A,
}

impl<A: Send> Dispatch<A> {
    /// Creates a new `Dispatch` message.
    pub fn new(action: A) -> Self {
        Self { action }
    }

    /// Consumes the message and returns the inner action.
    pub fn into_action(self) -> A {
        self.action
    }
}

impl<A: Send> Message for Dispatch<A> {
    type Response = ();
}

/// A `Select` message is used to query the state and retrieve derived data.
/// It wraps a selector function that is executed by the actor on the current state.
///
/// This message provides a safe and controlled way to access the state, ensuring that
/// the state itself is never directly exposed or modified by the querier.
pub struct Select<State, S>
where
    S: Selector<State>,
{
    selector: S,
    _types: PhantomData<State>,
}

impl<State, S> Select<State, S>
where
    S: Selector<State>,
{
    /// Creates a new `Select` message.
    pub fn new(selector: S) -> Self {
        Select {
            selector,
            _types: Default::default(),
        }
    }

    /// Consumes the message and returns the inner selector.
    pub fn into_selector(self) -> S {
        self.selector
    }
}

impl<State, S> Message for Select<State, S>
where
    State: Send,
    S: Selector<State> + Send,
    S::Result: Send,
{
    type Response = S::Result;
}