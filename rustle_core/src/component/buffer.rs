use ropey::Rope;
use taffy::{Dimension, Size, Style};

use crate::{
    component::root::State,
    input::Action,
    ui::{Color, Container, Element, TextSpan},
};

// TODO: Implement a buffer component as it is document in the legacy version.
// TODO: Rename buffer_view to window, access the buffer through the active window.

#[derive(Default)]
pub(crate) struct Buffer {
    text: Rope,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn reduce(buffer: Buffer, _action: Action) -> Buffer {
    buffer
}

pub(crate) fn render(_state: &State) -> Element {
    Element::Container(Box::new(Container {
        style: Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            flex_grow: 1.0,
            ..Default::default()
        },
        children: vec![Element::Span(TextSpan {
            background: Color::DarkGray,
            color: Color::White,
            text: String::new(),
        })],
    }))
}
