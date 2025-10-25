use taffy::{Dimension, FlexDirection, JustifyContent, Size, Style};

use crate::{
    component::{buffer_view, command_line},
    editor::State,
    ui::{Container, Element},
    Rect,
};

pub(crate) fn render(state: &State, area: Rect) -> Element {
    Element::Container(Box::new(Container {
        style: Style {
            size: Size {
                width: Dimension::length(f32::from(area.width)),
                height: Dimension::length(f32::from(area.height)),
            },
            flex_direction: FlexDirection::Column,
            justify_content: Some(JustifyContent::FlexEnd),
            ..Default::default()
        },
        children: vec![buffer_view::render(state), command_line::render(state)],
    }))
}
