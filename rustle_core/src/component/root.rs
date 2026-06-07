use taffy::{Dimension, FlexDirection, JustifyContent, Size, Style};

use crate::{
    component::{command_line, command_line::CommandLine, window},
    input::{Action, Mode},
    ui::{Container, Element},
    Position, Rect,
};

/// The `State` struct represents the state of the editor.
#[derive(Clone, Default)]
pub(crate) struct State {
    pub(crate) command_line: CommandLine,
    pub(crate) cursor_position: Position,
    pub(crate) mode: Mode,
    pub(crate) should_quit: bool,
}

/// The root `reduce` function is the main reducer for the editor.
/// It is responsible for handling all actions and updating the state.
// The `needless_pass_by_value` lint is allowed here because the function signature is constrained
// by the `Reducer` trait, which requires the `action` to be passed by value. This is a deliberate
// design choice that simplifies ownership and is efficient for small, `Copy`-like actions.
// Since our `Action` enum holds a `char`, which is a 4-byte primitive, the performance cost
// of passing by value is negligible. For a more detailed rationale, see the comments in
// the `Reducer` trait definition.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn reduce(mut state: State, action: Action) -> State {
    match action {
        Action::EnterMode(mode) => {
            state.mode = mode;
        }
        Action::Quit => state.should_quit = true,
        Action::Cancel => (),
    }

    if let Mode::Command = state.mode {
        state.command_line = command_line::reduce(state.command_line, &action);
    }

    if let Action::Cancel = action {
        state.mode = Mode::Normal;
    }

    state
}

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
        children: vec![window::render(state), command_line::render(state)],
    }))
}
