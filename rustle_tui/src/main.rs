#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Error};
use backtrace::Backtrace;
use crossterm::{style::Print, terminal::LeaveAlternateScreen};
use rustle_core::Editor;
use rustle_tui::{map_crossterm_event_stream, CrosstermCanvas};
use std::panic::{self, PanicInfo};

#[tokio::main]
async fn main() -> Result<(), Error> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    let mut canvas =
        CrosstermCanvas::new(std::io::stdout()).context("creating crossterm canvas")?;

    Editor::new(&mut canvas)
        .context("creating editor")?
        .consume(map_crossterm_event_stream())
        .await
        .context("consuming event stream")?;

    Ok(())
}

fn panic_hook(info: &PanicInfo<'_>) {
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
