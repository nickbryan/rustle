#![warn(clippy::all, clippy::pedantic)]

use std::{
    io,
    panic::{self, PanicHookInfo},
};

use anyhow::{Context, Error};
use backtrace::Backtrace;
use crossterm::{style::Print, terminal::LeaveAlternateScreen};
use rustle_core::Editor;

use crate::backend::CrosstermCanvas;

mod backend;

#[tokio::main]
async fn main() -> Result<(), Error> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    let canvas = CrosstermCanvas::new(io::stdout()).context("creating crossterm canvas")?;

    let (mut editor, mut actor) = Editor::new(canvas);

    tokio::spawn(async move { actor.act().await });

    editor
        .consume(backend::map_crossterm_event_stream())
        .await?;

    Ok(())
}

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
