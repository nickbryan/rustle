#![warn(clippy::all, clippy::pedantic)]

use rustle_core::Editor;
use rustle_tui::map_crossterm_event_stream;

#[tokio::main]
async fn main() {
    Editor::new()
        .consume(map_crossterm_event_stream())
        .await
        .unwrap(); //TODO: sort error
}
