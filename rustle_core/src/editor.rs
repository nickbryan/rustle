use std::ops::{Add, Deref};

use rustle_state::{ReducerFn, Runtime, StateError, Store};
use taffy::Style;
use tokio_stream::StreamExt;

use crate::{
    config::Config,
    error::Error,
    input::{Event, EventStream, Key},
    ui::{Canvas, Color, Component, Container, Element, TextSpan, Viewport},
};

/// The `Editor` struct represents the main component of the text editor application.
/// It encapsulates the entire state of the editor and provides the primary interface
/// for interacting with it.
///
/// The `Editor` holds an instance of the `Store`, which is responsible for managing
/// the application state. All state changes and queries are channeled through the
/// `Store`, ensuring a predictable and maintainable architecture.
pub struct Editor<C: Canvas> {
    state: Store<ReducerFn<State, Action>, State, Action>,
    canvas: C,
    _config: Config,
}

impl<C: Canvas> Editor<C> {
    pub fn new(canvas: C, runtime: &impl Runtime, config: Config) -> Self {
        Self {
            state: Store::new(
                root_reducer,
                State {
                    content: String::new(),
                    test: String::new(),
                    should_quit: false,
                },
                runtime,
            ),
            canvas,
            _config: config,
        }
    }

    /// Consume the given `EventStream` to run/drive the Editor.
    ///
    /// # Errors
    ///
    /// This function will return a `CoreError` if an unrecoverable error occurs.
    /// Possible error variants include:
    ///
    /// - `CoreError::Input`: If there is a failure reading from the event stream.
    /// - `CoreError::Ui`: If there is an error related to rendering the user interface.
    /// - `CoreError::State`: If a critical state management error occurs, such as the actor terminating unexpectedly.
    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<(), Error> {
        let mut viewport = Viewport::new(&mut self.canvas).map_err(Error::Ui)?;

        let mut state_rx = self.state.subscribe();

        while !self.state.select(|state: &State| state.should_quit).await? {
            tokio::select! {
                Some(event) = event_stream.next() => {
                    match event {
                        Event::KeyPressed(Key::Char(c)) => {
                            self.state.dispatch(Action::InsertChar(c)).await?;
                        }
                        Event::ReadFailed(e) => {
                            return Err(Error::Input(e.to_string()));
                        }
                        _ => (),
                    }
                }
                Ok(()) = state_rx.changed() => {
                    viewport.render(state_rx.borrow().deref(), &RootComponent)
                        .map_err(Error::Ui)?;
                }
                else => return Err(StateError::ActorTerminated.into()),
            }
        }

        Ok(())
    }
}

/// The `State` struct represents the state of the editor.
#[derive(Default, Clone, Debug, PartialEq)]
struct State {
    content: String,
    test: String,
    should_quit: bool,
}

/// The `Action` enum represents the actions that can be dispatched to the store.
enum Action {
    InsertChar(char),
}

/// The `root_reducer` is the main reducer for the editor.
/// It is responsible for handling all actions and updating the state.
// The `needless_pass_by_value` lint is allowed here because the function signature is constrained
// by the `Reducer` trait, which requires the `action` to be passed by value. This is a deliberate
// design choice that simplifies ownership and is efficient for small, `Copy`-like actions.
// Since our `Action` enum holds a `char`, which is a 4-byte primitive, the performance cost
// of passing by value is negligible. For a more detailed rationale, see the comments in
// the `Reducer` trait definition.
#[allow(clippy::needless_pass_by_value)]
fn root_reducer(mut state: State, action: Action) -> State {
    match action {
        Action::InsertChar('q') => state.should_quit = true,
        Action::InsertChar('a') => state.test.push('a'),
        Action::InsertChar('s') => {
            state.content = state.content.add(&state.test);
            state.test.clear();
        }
        Action::InsertChar(c) => state.content.push(c),
    }

    state
}

struct RootComponent;

#[derive(Clone, PartialEq)]
struct RootComponentProps {
    content: String,
}

impl Component<&State> for RootComponent {
    type Props = RootComponentProps;

    fn select(&self, state: &State) -> Self::Props {
        // The `content` string is cloned here. While this is not ideal for performance,
        // it is a necessary trade-off for the current architecture. The rendering engine
        // (`TextSpan`, `render_element`, etc.) is designed to work with owned `String`s
        // for simplicity. Removing this clone would require a significant architectural
        // refactoring to use string slices (`&str`) and lifetimes throughout the UI
        // rendering code. This is a potential future optimization if performance becomes
        // a bottleneck.
        RootComponentProps {
            content: state.content.clone(),
        }
    }

    fn render(&self, props: Self::Props) -> Element {
        Element::Container(Box::new(Container {
            layout: Style::default(),
            children: vec![
                Element::Span(TextSpan {
                    background: Color::DarkGray,
                    color: Color::Yellow,
                    text: props.content.clone(),
                }),
                Element::Span(TextSpan {
                    background: Color::DarkGray,
                    color: Color::Yellow,
                    text: props.content,
                }),
            ],
        }))
    }
}
