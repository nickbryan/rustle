use crate::state::Store;

/// The `Editor` struct represents the main component of the text editor application.
/// It encapsulates the entire state of the editor and provides the primary interface
/// for interacting with it.
///
/// The `Editor` holds an instance of the `Store`, which is responsible for managing
/// the application state. All state changes and queries are channeled through the
/// `Store`, ensuring a predictable and maintainable architecture.
pub struct Editor {
    state: Store<fn(State, Action) -> State, State, Action>,
}

impl Editor {
    /// Creates a new editor.
    pub fn new() -> Self {
        let editor = Self {
            state: Store::new(root_reducer),
        };

        editor.state.subscribe(move |state| {
            println!("State update: {:?}", state);
        });

        editor
    }
}

/// The `root_reducer` is the main reducer for the editor.
/// It is responsible for handling all actions and updating the state.
fn root_reducer(state: State, action: Action) -> State {
    state
}

/// The `State` struct represents the state of the editor.
#[derive(Default, Clone, Debug, PartialEq)]
struct State{
    content: String,
}

/// The `Action` enum represents the actions that can be dispatched to the store.
enum Action {
    InsertChar(char),
}