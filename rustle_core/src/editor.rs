use std::{ops::Deref, time::Duration};

use rustle_state::{ReducerFn, Runtime, StateError, Store};
use tokio::time;
use tokio_stream::StreamExt;

use crate::{
    component,
    component::root::State,
    config::Config,
    error::Error,
    input::{Action, Event, EventStream, Mode, Resolution, Resolver},
    ui::{Canvas, Viewport},
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
    resolver: Resolver,
    state: Store<ReducerFn<State, Action>, State, Action>,
    idle_timeout: Duration,
}

impl<C: Canvas> Editor<C> {
    pub fn new(config: Config, canvas: C, runtime: &impl Runtime) -> Self {
        Self {
            canvas,
            resolver: Resolver::new(config.bindings),
            state: Store::new(component::root::reduce, State::default(), runtime),
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
        let mut pending_timeout = false;

        while !self.state.select(|state: &State| state.should_quit).await? {
            let timeout = if pending_timeout {
                self.idle_timeout
            } else {
                Duration::from_secs(u64::MAX)
            };

            tokio::select! {
                // TODO: This needs to work with both the tokio and wasm runtimes for web and terminal.
                result = time::timeout(timeout, event_stream.next()) => match result {
                    Ok(Some(Event::KeyPressed(key))) => {
                        let mode = self.state.select(|state: &State| state.mode).await?;
                        let resolution = self.resolver.resolve(key, mode);

                        match resolution {
                            Resolution::Match(action) => {
                                pending_timeout = false;
                                self.state.dispatch(action).await?;
                            }
                            Resolution::Pending => {
                                if mode == Mode::Insert {
                                    pending_timeout = true;
                                }
                            }
                            Resolution::NoMatch => {
                                if mode == Mode::Insert {
                                    // TODO: add these back when the Actions work.
                                    // let text = self.resolver.drain_buffer();
                                    // self.state.dispatch(Action::InsertString(text)).await?;
                                }
                                self.resolver.reset();
                                pending_timeout = false;
                            }
                        }
                    }
                    Err(_) => { // Timeout.
                        // TODO: add these back when the Actions work.
                        // let text = self.resolver.drain_buffer();
                        // self.state.dispatch(Action::InsertString(text)).await?;
                        self.resolver.reset();
                        pending_timeout = false;
                    }
                    _ => (),
                },
                Ok(()) = state_rx.changed() => {
                    viewport.redraw(state_rx.borrow().deref(), component::root::render).map_err(Error::Ui)?;
                }
                else => return Err(StateError::ActorTerminated.into()),
            }
        }

        Ok(())
    }
}
