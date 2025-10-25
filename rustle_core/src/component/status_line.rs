use taffy::prelude::*;

use crate::{
    editor::State,
    ui::{Color, Container, Element, TextSpan},
};

pub(crate) fn render(state: &State) -> Element {
    Element::Container(Box::new(Container {
        style: Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::length(1.0),
            },
            ..Default::default()
        },
        children: vec![Element::Span(TextSpan {
            background: Color::DarkGray,
            color: Color::Yellow,
            text: state.mode.to_string() + " ",
        })],
    }))
}
