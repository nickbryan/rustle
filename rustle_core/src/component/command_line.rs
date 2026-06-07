use taffy::prelude::*;

use crate::{
    component::root::State,
    input::Action,
    ui::{Color, Container, Element, TextSpan},
};

#[derive(Default, Clone)]
pub(crate) struct CommandLine {
    text: String,
}

pub(crate) fn reduce(mut command_line: CommandLine, action: &Action) -> CommandLine {
    if let Action::Cancel = action {
        command_line.text.clear();
        return command_line;
    }

    command_line.text = String::from(":");

    command_line
}

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
            text: state.command_line.text.clone(),
        })],
    }))
}
