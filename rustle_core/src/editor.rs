use anyhow::{Error, Result};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::{
    component::Window,
    mode,
    mode::Normal,
    render::{View, Viewport},
    Canvas, Event, EventStream, Mode,
};

// TODO: write tests for existing code
// TODO: review and refactor all code for correctness
// TODO: refactor storage to be abstract so that it works on web and terminal.
// TODO: convert line numbers into a widget
// TODO: implement selections

/// `Command` is an enum that captures all commands that the `Editor` and its `Component`s
/// understand.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    BatchExecute(Vec<Command>),

    AbortCommandLineInput,
    EndCommandLineInput,
    ParseCommandLineInput(String),

    EnterMode(Mode),

    InsertChar(char),
    InsertLineBreak,
    DeleteCharForward,
    DeleteCharBackward,

    MoveCursorUp(usize),
    MoveCursorDown(usize),
    MoveCursorLeft(usize),
    MoveCursorRight(usize),
    MoveCursorLineStart,
    MoveCursorLineEnd,
    MoveCursorPageUp,
    MoveCursorPageDown,

    Open(String),
    Save,
    SaveAs(String),

    BufferNext,
    BufferPrevious,

    VisualSplit,

    PreviousWindow,

    Quit,
}

/// `Component` is the foundation for all interactivity within the `Editor`. You can view it as the
/// model in elm architecture.
pub trait Component {
    fn update(&mut self, cmd: Command) -> Result<Option<Command>>;
}

/// `Editor` is the entry point into the application and is responsible for orchestrating
/// communication between `Component`s.
pub struct Editor<'a, VC, C>
where
    VC: View + Component,
    C: Canvas,
{
    mode: Mode,
    root_component: VC,
    should_quit: bool,
    viewport: Viewport<'a, C>,
}

impl<'a, C> Editor<'a, Window, C>
where
    C: Canvas,
{
    /// Create a new editor using the default `View` `Component` and the given `Canvas`.
    ///
    /// # Errors
    ///
    /// Can error while creating the `Viewport` if the underlying `Canvas` has IO issues.
    pub fn new(canvas: &'a mut C) -> Result<Self> {
        use anyhow::Context;

        let mode = Mode::default();
        let viewport = Viewport::new(canvas).context("unable to initialise Viewport")?;

        Ok(Self {
            mode: mode.clone(),
            root_component: Window::new(viewport.area(), mode),
            should_quit: false,
            viewport,
        })
    }
}

impl<'a, VC, C> Editor<'a, VC, C>
where
    VC: Component + View,
    C: Canvas,
{
    /// Consume the given `EventStream` to run/drive the Editor.
    ///
    /// # Errors
    /// Will return `Err` when a message was received on the `err_tx`.
    ///
    /// # Panics
    /// When the command channels are closed unexpectedly.
    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<()> {
        use anyhow::Context;

        let (err_tx, mut err_rx) = mpsc::channel::<Error>(1);
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();

        // Render the initial view so that we don't have to wait for an input event to
        // see something on the screen.
        self.viewport
            .render(&self.root_component)
            .context("unable to render the initial view")?;

        while !self.should_quit {
            tokio::select! {
                Some(event) = event_stream.next() => {
                    match event {
                        Event::KeyPressed(key) => {
                            if let Some(cmd) = match self.mode {
                                Mode::Execute => mode::Execute::handle(key),
                                Mode::Insert => mode::Insert::handle(key),
                                Mode::Normal(ref mut mode) => mode.handle(key),
                            } {
                                cmd_tx
                                    .send(cmd)
                                    .expect("unable to send cmd on closed cmd_tx channel");
                            }
                        }
                        Event::ReadFailed(e) => {
                            err_tx
                                .send(Error::new(e))
                                .await
                                .expect("unable to send on closed err_tx channel");
                        }
                        _ => (),
                    }
                }
                Some(e) = err_rx.recv() => {
                    return Err(e);
                }
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        Command::Quit => self.should_quit = true,
                        Command::EnterMode(ref mode) => self.mode = mode.clone(),
                        Command::ParseCommandLineInput(input) => {
                            if let Mode::Execute = self.mode {
                                if let Some(c) = mode::Execute::parse(&input) {
                                     cmd_tx
                                    .send(c)
                                    .expect("unable to send cmd on closed cmd_tx channel");
                                }
                            }

                            cmd_tx
                                .send(Command::EnterMode(Mode::Normal(Normal::default())))
                                .expect("unable to send cmd on closed cmd_tx channel");

                            continue;
                        }
                        Command::BatchExecute(commands) => {
                            for command in commands {
                                cmd_tx
                                    .send(command)
                                    .expect("unable to send command on closed cmd_tx channel");
                            }

                            continue;
                        }
                        _ => (),
                    };

                    match self.root_component.update(cmd) {
                        Ok(Some(cmd)) => {
                            cmd_tx.send(cmd).expect("unable to send on closed cmd_tx channel");
                        }
                        Err(e) => {
                            err_tx.send(e.context("error during root_component update")).await.expect("unable to send on closed err_tx channel");
                        }
                        _ => (),
                    }

                    if let Err(e) = self.viewport.render(&self.root_component).context("rendering error occurred") {
                        err_tx.send(e).await.expect("unable to send on closed err_tx channel");
                    }
                }
                else => break,
            }
        }

        Ok(())
    }
}
