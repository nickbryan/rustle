#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Error};
use rustle_core::Editor;
use rustle_tui::{map_crossterm_event_stream, CrosstermCanvas};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut canvas =
        CrosstermCanvas::new(std::io::stdout()).context("creating crossterm canvas")?;

    Editor::new(&mut canvas)
        .context("creating editor")?
        .consume(map_crossterm_event_stream())
        .await
        .context("consuming event stream")?;

    Ok(())
}
