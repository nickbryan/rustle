use std::ops::{Add, Deref};
use taffy::{LengthPercentageAuto, Position, Rect, Style};
use tokio_stream::StreamExt;
use crate::input::{Event, EventStream, Key};
use crate::state::Store;
use crate::ui::component::{_Component, _Container, _Element, _TextSpan};
use crate::ui::render::{Canvas, Viewport};
use crate::ui::values::Color;

/// The `Editor` struct represents the main component of the text editor application.
/// It encapsulates the entire state of the editor and provides the primary interface
/// for interacting with it.
///
/// The `Editor` holds an instance of the `Store`, which is responsible for managing
/// the application state. All state changes and queries are channeled through the
/// `Store`, ensuring a predictable and maintainable architecture.
pub struct Editor<C: Canvas> {
    state: Store<fn(State, Action) -> State, State, Action>,
    canvas: C,
}

impl<C: Canvas + Send + Sync> Editor<C> {
    pub fn new(canvas: C) -> Self {
            Self {
                state: Store::new(root_reducer, State {
                    content: "".to_string(),
                    should_quit: false,
                }),
                canvas
            }
    }

    /// Consume the given `EventStream` to run/drive the Editor.
    ///
    /// # Errors
    /// Will return `Err` when a message was received on the `err_tx`.
    ///
    /// # Panics
    /// When the command channels are closed unexpectedly.
    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<(), ()> {
        let mut viewport = Viewport::new(&mut self.canvas).unwrap(); // TODO: handle error.
        let mut state_rx = self.state.subscribe();

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
                Ok(_) = state_rx.changed() => {
                    viewport.render(state_rx.borrow().deref(), RootComponent).unwrap();
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
            state.content.push(c);
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

struct RootComponent;

struct RootComponentProps {
    content: String,
}

impl _Component<&State> for RootComponent {
    type Props = RootComponentProps;

    fn select(&self, state: &State) -> Self::Props {
        RootComponentProps{content: state.content.clone() } // TODO: can we get rid of this clone?
    }

    fn render(&self, props: Self::Props) -> _Element {
        _Element::Container(_Container{
            layout: Style{
                padding: Rect::length(5.0),
                ..Default::default()
            },
            children: vec![
                _Element::Span(_TextSpan{
                    background: Color::White,
                    color: Color::Blue,
                    text: props.content,
                })
            ],
        })
    }
}