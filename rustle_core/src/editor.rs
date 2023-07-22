use crate::input::EventStream;
use crate::Event;
use anyhow::{Error, Result};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub async fn consume(&mut self, mut event_stream: EventStream) -> Result<()> {
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
                                .expect("the msg_tx channel should not be closed");
                        }
                        Event::ReadFailed(e) => {
                            err_tx
                                .send(Error::new(e))
                                .await
                                .expect("the err_tx channel should not be closed");
                        }
                        _ => (),
                    }
                }
                Some(key) = msg_rx.recv() => {
                        println!("{:?}", key);
                        self.should_quit = true;
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
