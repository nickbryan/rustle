use std::ops::Add;
use tokio_stream::StreamExt;
use crate::input::{Event, EventStream, Key};
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

impl Default for Editor {
    fn default() -> Self {
        Self {
            state: Store::new(root_reducer, State {
                content: "".to_string(),
                should_quit: false,
            }),
        }
    }
}

impl Editor {
    /// Consume the given `EventStream` to run/drive the Editor.
    ///
    /// # Errors
    /// Will return `Err` when a message was received on the `err_tx`.
    ///
    /// # Panics
    /// When the command channels are closed unexpectedly.
    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<(), ()> {
        self.state.subscribe(move |state| {
            println!("State update: {:?}", state);
        });

        while !self.state.select(|state: &State| state.should_quit).await {
            tokio::select! {
                Some(event) = event_stream.next() => {
                    match event {
                        Event::KeyPressed(Key::Char(c)) => {
                            self.state.dispatch(Action::InsertChar(c)).await;
                        }
                        Event::ReadFailed(_) => {
                            return Err(())
                        }
                        _ => (),
                    }
                }
                else => return Err(()),
            }
        }

        Ok(())
    }
}

/// The `root_reducer` is the main reducer for the editor.
/// It is responsible for handling all actions and updating the state.
fn root_reducer(mut state: State, action: Action) -> State {
    match action {
        Action::InsertChar('q') => {
            state.should_quit = true;
        },
        Action::InsertChar(c) => {
            state.content = state.content.add(c.to_string().as_str());
        }
    }

    state
}

/// The `State` struct represents the state of the editor.
#[derive(Default, Clone, Debug, PartialEq)]
struct State{
    content: String,
    should_quit: bool,
}

/// The `Action` enum represents the actions that can be dispatched to the store.
enum Action {
    InsertChar(char),
}