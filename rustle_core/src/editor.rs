use crate::state::{Store};

#[derive(Default)]
struct State {}

enum Action {}

struct Editor {
    state: Store<fn(State, Action) -> State, State, Action>,
}

impl Editor {
    fn new() -> Self {
        Self {
            state: Store::new(root_reducer),
        }
    }
}

fn root_reducer(state: State, _: Action) -> State {
    state
}