use crate::communication;
use crate::communication::{Command, Message};
use crate::component::{Compositor, TextInput};
use crate::editor::Component;
use crate::mode::Mode;
use crate::render::{Frame, View};
use crate::ui::Rect;
use anyhow::{Context, Result};
use taffy::prelude::*;

/// `Compositor` is the default root component for the `Editor`.
pub struct Window {
    command_prompt: TextInput,
    compositor: Compositor,
    mode: Mode,
}

impl Window {
    pub fn new(size: Rect, mode: Mode) -> Self {
        let mut taffy = Taffy::new();

        let body_node = taffy
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Auto,
                },
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();

        let command_node = taffy
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Points(1.0),
                },
                ..Default::default()
            })
            .unwrap();

        let root_node = taffy
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: Some(JustifyContent::FlexEnd),
                    size: Size {
                        width: Dimension::Points(f32::from(size.width)),
                        height: Dimension::Points(f32::from(size.height)),
                    },
                    ..Default::default()
                },
                &[body_node, command_node],
            )
            .unwrap();

        taffy
            .compute_layout(
                root_node,
                Size {
                    height: AvailableSpace::Definite(f32::from(size.width)),
                    width: AvailableSpace::Definite(f32::from(size.height)),
                },
            )
            .unwrap();

        let mut command_prompt = TextInput::new(
            ":",
            " Press : to enter a command...",
            taffy.layout(command_node).unwrap().location.into(),
        );

        if let Mode::Execute = mode {
            command_prompt.focus();
        }

        Self {
            command_prompt,
            compositor: Compositor::new(taffy.layout(body_node).unwrap().into(), mode.clone()),
            mode,
        }
    }
}

impl Component for Window {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        let mut commands = vec![];

        if let Message::EnterMode(ref mode) = msg {
            if let Mode::Execute = mode {
                self.command_prompt.focus();
            } else {
                self.command_prompt.unfocus();
            }

            self.mode = mode.clone();
        }

        if let Mode::Execute = self.mode {
            commands.push(
                self.command_prompt
                    .update(msg.clone())
                    .context("updating command prompt")?,
            );
        }

        commands.push(self.compositor.update(msg).context("updating compositor")?);

        Ok(Some(communication::batch(commands)))
    }
}

impl View for Window {
    fn render_to(&self, frame: &mut Frame) {
        self.compositor.render_to(frame);
        self.command_prompt.render_to(frame);
    }
}
