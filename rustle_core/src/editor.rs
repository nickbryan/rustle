use std::{ops::Deref, time::Duration};

use rustle_state::{ReducerFn, Runtime, StateError, Store};
use taffy::Style;
use tokio::time;
use tokio_stream::StreamExt;

use crate::{
    config::Config,
    error::Error,
    input::{Event, EventStream, Mode, Processor},
    ui::{Canvas, Color, Component, Container, Element, TextSpan, Viewport},
    Position,
};

/// The `Editor` struct represents the core of the text editor application.
/// It encapsulates the entire state of the editor and provides the primary interface
/// for interacting with it.
///
/// The `Editor` holds an instance of the `Store`, which is responsible for managing
/// the application state. All state changes and queries are channeled through the
/// `Store`, ensuring a predictable and maintainable architecture.
pub struct Editor<C: Canvas> {
    canvas: C,
    processor: Processor,
    state: Store<ReducerFn<State, Action>, State, Action>,
    idle_timeout: Duration,
}

impl<C: Canvas> Editor<C> {
    pub fn new(config: Config, canvas: C, runtime: &impl Runtime) -> Self {
        Self {
            canvas,
            processor: Processor::new(config.bindings),
            state: Store::new(root_reducer, State::default(), runtime),
            idle_timeout: Duration::from_millis(config.editor.idle_timeout),
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
                result = time::timeout(self.idle_timeout, event_stream.next()) => match result {
                    Ok(Some(event)) => match event {
                        Event::KeyPressed(key) => {
                            if let Some(action) = self.processor.process(
                                key,
                                self.state.select(|state: &State| state.mode).await?
                            ) {
                                self.state.dispatch(action).await?;
                            }
                        }
                        Event::ReadFailed(e) => {
                            return Err(Error::Input(e.to_string()));
                        }
                        _ => (),
                    },
                    Err(_) => {
                        self.processor.clear();
                    }
                      _ => (),
                },
                Ok(()) = state_rx.changed() => {
                    viewport.render(state_rx.borrow().deref(), &RootComponent).map_err(Error::Ui)?;
                }
                else => return Err(StateError::ActorTerminated.into()),
            }
        }

        Ok(())
    }
}

/// The `State` struct represents the state of the editor.
#[derive(Default, Clone, PartialEq)]
struct State {
    cursor_position: Position,
    mode: Mode,
    should_quit: bool,
}

/// The `Action` enum represents the actions that can be dispatched to the store.
pub enum Action {
    EnterMode(Mode),
    MoveCursor(Movement),
    Quit,
}

#[derive(Clone, Copy)]
pub enum Movement {
    Next(u16),
    Prev(u16),
    LineNext(u16),
    LinePrev(u16),
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
        Action::EnterMode(mode) => state.mode = mode,
        Action::Quit => state.should_quit = true,
        Action::MoveCursor(movement) => {
            state = cursor_position_reducer(state, movement);
        }
    }

    state
}

fn cursor_position_reducer(mut state: State, movement: Movement) -> State {
    match movement {
        Movement::Next(chars) => {
            state.cursor_position.col = state.cursor_position.col.saturating_add(chars);
        }
        Movement::Prev(chars) => {
            state.cursor_position.col = state.cursor_position.col.saturating_sub(chars);
        }
        Movement::LineNext(lines) => {
            state.cursor_position.row = state.cursor_position.row.saturating_add(lines);
        }
        Movement::LinePrev(lines) => {
            state.cursor_position.row = state.cursor_position.row.saturating_sub(lines);
        }
    }

    state
}

struct RootComponent;

#[derive(Clone, PartialEq)]
struct RootComponentProps {
    mode: String,
    cursor_position: Position,
}

impl Component<&State> for RootComponent {
    type Props = RootComponentProps;

    fn select(&self, state: &State) -> Self::Props {
        RootComponentProps {
            mode: state.mode.to_string(),
            cursor_position: state.cursor_position,
        }
    }

    fn render(&self, props: Self::Props) -> Element {
        Element::Container(Box::new(Container {
            layout: Style::default(),
            children: vec![Element::Span(TextSpan {
                background: Color::DarkGray,
                color: Color::Yellow,
                text: props.mode,
            })],
            cursor_position: props.cursor_position,
        }))
    }
}
