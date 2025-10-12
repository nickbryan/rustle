#![warn(clippy::all, clippy::pedantic)]

use std::{
    future::Future,
    io,
    panic::{self, PanicHookInfo},
};

use anyhow::{Context, Error};
use backtrace::Backtrace;
use crossterm::{style::Print, terminal::LeaveAlternateScreen};
use rustle_core::{Config, Editor};
use rustle_state::Runtime;

use crate::backend::CrosstermCanvas;

mod backend;

/// A `Runtime` implementation for the terminal environment.
/// It uses `tokio::spawn` to spawn tasks.
struct TokioRuntime;

impl Runtime for TokioRuntime {
    /// Spawns a new future on the Tokio runtime.
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(future);
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up a custom panic hook to ensure that the terminal is restored to a
    // usable state in the event of a panic. This is crucial for preventing the
    // terminal from being left in a raw mode, which would make it difficult to
    // use.
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    // Create a new `CrosstermCanvas`, which is an implementation of the `Canvas`
    // trait that uses `crossterm` to render the UI in the terminal.
    let canvas = CrosstermCanvas::new(io::stdout()).context("creating crossterm canvas")?;

    // Create a new `Editor` instance, which is the main component of the application.
    let mut editor = Editor::new(canvas, &TokioRuntime, Config::default());

    // Start the editor's event loop by consuming the event stream. The event
    // stream is a stream of input events from the terminal, such as key
    // presses. The `consume` method will block until the editor exits.
    editor
        .consume(backend::map_crossterm_event_stream())
        .await?;

    Ok(())
}

/// A custom panic hook that restores the terminal to a usable state before
/// printing the panic message. This is essential for ensuring that the user
/// can still interact with their terminal after the application crashes.
fn panic_hook(info: &PanicHookInfo<'_>) {
    let location = info.location().unwrap();

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

    // Restore the terminal to a usable state.
    crossterm::terminal::disable_raw_mode().expect("unable to disable raw mode");
    crossterm::execute!(
        std::io::stdout(),
        LeaveAlternateScreen,
        Print(format!(
            "thread '<unnamed>' panicked at '{msg}', {location}\n\r{stacktrace}"
        )),
    )
    .unwrap();
}
