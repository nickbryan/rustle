use crate::communication::{Command, Message};
use crate::component::Window;
use crate::input::EventStream;
use crate::render::{Canvas, View, Viewport};
use crate::Event;
use anyhow::{Context, Error, Result};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// `Component` is the foundation for all interactivity within the `Editor`. You can view it as the
/// model in elm architecture.
pub trait Component {
    fn update(&mut self, msg: Message) -> Result<Option<Command>>;
}

pub struct Editor<'a, C, VC>
where
    C: Canvas,
    VC: View + Component,
{
    root_component: VC,
    should_quit: bool,
    viewport: Viewport<'a, C>,
}

impl<'a, C> Editor<'a, C, Window>
where
    C: Canvas,
{
    pub fn new(canvas: &'a mut C) -> Result<Self> {
        let viewport = Viewport::new(canvas).context("creating viewport")?;

        Ok(Self {
            root_component: Window::new(viewport.area()),
            should_quit: false,
            viewport,
        })
    }

    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<()> {
        // Render the initial view so that we don't have to wait for an input event to
        // see something on the screen.
        self.viewport
            .render(&self.root_component)
            .context("rendering the initial view")?;

        let (err_tx, mut err_rx) = mpsc::channel::<Error>(1);
        let (msg_tx, mut msg_rx) = mpsc::channel(1);

        while !self.should_quit {
            tokio::select! {
                Some(event) = event_stream.next() => {
                    match event {
                        Event::KeyPressed(key) => {
                            msg_tx
                                .send(key)
                                .await
                                .expect("the msg_tx channel should not be closed when a key is pressed");
                        }
                        Event::ReadFailed(e) => {
                            err_tx
                                .send(Error::new(e))
                                .await
                                .expect("the err_tx channel should not be closed when reading failed");
                        }
                        _ => (),
                    }
                }
                Some(key) = msg_rx.recv() => {
                    if let Err(e) = self.viewport.render(&self.root_component).context("rendering viewport") {
                        err_tx.send(e).await.expect("the err_tx channel should not be closed when rendering viewport");
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
