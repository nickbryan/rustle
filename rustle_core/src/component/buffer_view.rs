use taffy::{Dimension, FlexDirection, JustifyContent, Size, Style};

use crate::{
    component::{buffer, status_line},
    editor::State,
    ui::{Container, Element},
};

pub(crate) fn render(state: &State) -> Element {
    Element::Container(Box::new(Container {
        style: Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: Some(JustifyContent::FlexEnd),
            ..Default::default()
        },
        children: vec![buffer::render(state), status_line::render(state)],
    }))
}
