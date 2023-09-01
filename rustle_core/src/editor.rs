use crate::communication::{Command, Message};
use crate::component::Compositor;
use crate::mode::Normal;
use crate::render::{View, Viewport};
use crate::{mode, Canvas, Event, EventStream, Mode};
use anyhow::{Error, Result};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

// TODO: figure out grapheme handling, it is based on chars, 2 chars = 1 flag. Need to figure out how
// the selection, range and movement will work with grapheme boundaries. Is this the documents job,
// should it control the cursor?
// TODO: refactor the window component to use layouts. Lift the work of window into editor and have
// editor use the window API to open buffers etc.
// TODO: refactor storage to be abstract so that it works on web and terminal.
// TODO: convert line numbers into a widget
// TODO: review and refactor all code for correctness
// TODO: write tests for existing code
// TODO: implement selections

/// `Component` is the foundation for all interactivity within the `Editor`. You can view it as the
/// model in elm architecture.
pub trait Component {
    fn update(&mut self, msg: Message) -> Result<Option<Command>>;
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

impl<'a, C> Editor<'a, Compositor, C>
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
            root_component: Compositor::new(viewport.area(), mode),
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
    /// When the message channels are closed unexpectedly.
    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<()> {
        use anyhow::Context;

        let (err_tx, mut err_rx) = mpsc::channel::<Error>(1);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(1);
        // TODO: this had to be increased because of the two calls in ParseCommandLineInput.
        // Why does it need a buffer size? Should it not be async? What is causing it to block?
        let (msg_tx, mut msg_rx) = mpsc::channel(2);

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
                            if let Some(msg) = match self.mode {
                                Mode::Execute => mode::Execute::handle(key),
                                Mode::Insert => mode::Insert::handle(key),
                                Mode::Normal(ref mut mode) => mode.handle(key),
                            } {
                                msg_tx
                                    .send(msg)
                                    .await
                                    .expect("unable to send msg on closed msg_tx channel");
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
                Some(cmd) = cmd_rx.recv() => {
                    let msg_tx = msg_tx.clone();
                    // Each command is spawned in its own async block as they may take time to complete.
                    tokio::spawn(async move {
                        msg_tx
                            .send(cmd())
                            .await
                            .expect("unable to send cmd result on closed msg_tx channel");
                    });
                }
                Some(msg) = msg_rx.recv() => {
                    if let Message::Quit = msg {
                        self.should_quit = true;
                    }

                    if let Message::EnterMode(mode) = msg.clone() {
                        self.mode = mode;
                    }

                    if let Message::ParseCommandLineInput(input) = msg {
                        if let Mode::Execute = self.mode {
                            let msg = mode::Execute::parse(&input);

                            if let Some(msg) = msg {
                                 msg_tx
                                .send(msg)
                                .await
                                .expect("unable to send msg on closed msg_tx channel");
                            }
                        }

                        msg_tx
                                .send(Message::EnterMode(Mode::Normal(Normal::default())))
                                .await
                                .expect("unable to send msg on closed msg_tx channel");

                        continue;
                    }

                    match self.root_component.update(msg) {
                        Ok(Some(cmd)) => {
                            cmd_tx.send(cmd).await.expect("unable to send on closed cmd_tx channel");
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
                Some(e) = err_rx.recv() => {
                    return Err(e);
                }
                else => break,
            }
        }

        Ok(())
    }
}
