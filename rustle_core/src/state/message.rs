use std::marker::PhantomData;
use crate::{ 
    state::{ 
        mailbox::Message,
        selector::Selector
    }
};

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
    type Response = ();
}

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
    pub fn new(selector: S) -> Self {
        Select {
            selector,
            _types: Default::default(),
        }
    }

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