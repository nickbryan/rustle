use crate::communication::{Command, Message};
use crate::component::document::Document;
use crate::component::{StatusBar, TextInput, Welcome};
use crate::editor::Component;
use crate::mode::Mode;
use crate::render::{Frame, View};
use crate::ui::{Position, Rect};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use taffy::prelude::*;

/// `Compositor` is the default root component for the `Editor`.
pub struct Compositor {
    active_document_idx: usize,
    documents: Vec<(String, Document)>,
    document_name_indexes: HashMap<String, usize>,
    command_prompt: TextInput,
    mode: Mode,
    layout: Taffy,
    body_node: Node,
    status_node: Node,
}

impl Compositor {
    pub fn new(size: Rect, mode: Mode) -> Self {
        let mut layout = Taffy::new();

        let body_node = layout
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Auto,
                },
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();

        let status_node = layout
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Points(1.0),
                },
                ..Default::default()
            })
            .unwrap();

        let command_node = layout
            .new_leaf(Style {
                size: Size {
                    width: Dimension::Auto,
                    height: Dimension::Points(1.0),
                },
                ..Default::default()
            })
            .unwrap();

        let root_node = layout
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
                &[body_node, status_node, command_node],
            )
            .unwrap();

        layout
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
            layout.layout(command_node).unwrap().location.into(),
        );

        if let Mode::Execute = mode {
            command_prompt.focus();
        }

        Self {
            active_document_idx: 0,
            documents: Vec::default(),
            document_name_indexes: HashMap::new(),
            command_prompt,
            mode,
            layout,
            body_node,
            status_node,
        }
    }

    fn buffer_space(&self) -> Rect {
        self.layout.layout(self.body_node).unwrap().into()
    }
}

impl Component for Compositor {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        if let Message::EnterMode(mode) = msg.clone() {
            if let Mode::Insert = mode {
                if self.documents.is_empty() {
                    self.documents
                        .push((String::from("scratch"), Document::new(self.buffer_space())));
                }
            }

            if let Mode::Execute = mode {
                self.command_prompt.focus();
            } else {
                self.command_prompt.unfocus();
            }

            self.mode = mode;
        }

        if let Message::Open(path) = msg.clone() {
            if !self.document_name_indexes.contains_key(&path) {
                self.documents.push((
                    path.clone(),
                    Document::from(
                        self.buffer_space(),
                        &mut io::BufReader::new(
                            File::open(path.clone().as_str()).context("opening file")?,
                        ),
                    )
                    .context("opening document")?,
                ));
                self.document_name_indexes
                    .insert(path.clone(), self.documents.len() - 1);
            }

            self.active_document_idx = *self.document_name_indexes.get(&path).unwrap();
        }

        if let Message::BufferPrevious = msg.clone() {
            self.active_document_idx = self.active_document_idx.saturating_sub(1);
        }

        if let Message::BufferNext = msg.clone() {
            self.active_document_idx = self
                .active_document_idx
                .saturating_add(1)
                .min(self.documents.len().saturating_sub(1));
        }

        if let Mode::Execute = self.mode {
            return self.command_prompt.update(msg);
        }

        if !self.documents.is_empty() {
            return self.documents[self.active_document_idx].1.update(msg);
        }

        Ok(None)
    }
}

impl View for Compositor {
    fn render_to(&self, frame: &mut Frame) {
        if self.documents.is_empty() {
            Welcome {
                size: self.buffer_space(),
            }
            .render_to(frame);
        } else {
            self.documents[self.active_document_idx].1.render_to(frame);
        }

        let mut len = 0;

        if let Mode::Normal(_) | Mode::Insert = self.mode {
            frame.set_cursor_position(if self.documents.is_empty() {
                self.layout.layout(self.body_node).unwrap().location.into()
            } else {
                let body_position: Position =
                    self.layout.layout(self.body_node).unwrap().location.into();
                Position::new(
                    self.documents[self.active_document_idx]
                        .1
                        .cursor_position()
                        .col
                        .saturating_add(body_position.col),
                    self.documents[self.active_document_idx]
                        .1
                        .cursor_position()
                        .row
                        .saturating_add(body_position.row),
                )
            });
        }

        if !self.documents.is_empty() {
            len = self.documents[self.active_document_idx].1.len();
        }

        StatusBar {
            area: self.layout.layout(self.status_node).unwrap().into(),
            mode: self.mode.to_string(),
            line_count: len,
            cursor_position: frame.cursor_position(), // TODO: not accounting for margin.
            file_name: self
                .documents
                .get(self.active_document_idx)
                .unwrap_or(&(String::new(), Document::default()))
                .0
                .clone(),
        }
        .render_to(frame);

        self.command_prompt.render_to(frame);
    }
}
