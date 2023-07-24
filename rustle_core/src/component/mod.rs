use crate::communication::{Command, Message};
use anyhow::Result;

mod buffer;
mod status_bar;
mod text_input;
mod welcome;
mod window;

use buffer::Buffer;
use status_bar::StatusBar;
use text_input::TextInput;
use welcome::Welcome;
pub use window::Window;

/// `Component` is the foundation for all interactivity within the `Editor`. You can view it as the
/// model in elm architecture.
pub trait Component {
    fn update(&mut self, msg: Message) -> Result<Option<Command>>;
}
