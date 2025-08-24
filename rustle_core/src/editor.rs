use crate::state::Store;

struct Editor {
    state: Store<fn(State, Action) -> State, State, Action>,
}

impl Editor {
    fn new() -> Self {
        let editor = Self {
            state: Store::new(root_reducer),
        };

        editor.state.subscribe(move |state| {
            println!("State update: {:?}", state);
        });

        editor
    }
}

fn root_reducer(state: State, action: Action) -> State {
    state
}

#[derive(Default, Clone, Debug, PartialEq)]
struct State{
    content: String,
}

enum Action {
    InsertChar(char),
}