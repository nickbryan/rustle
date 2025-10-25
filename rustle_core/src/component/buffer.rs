use taffy::{Dimension, Size, Style};

use crate::{
    editor::State,
    ui::{Color, Container, Element, TextSpan},
};

pub(crate) fn render(state: &State) -> Element {
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
            text: state.buffer.to_string(),
        })],
    }))
}
