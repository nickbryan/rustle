use crate::communication::{Command, Message};
use crate::component::document::Document;
use crate::component::Welcome;
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
    documents: Vec<Document>,
    document_name_indexes: HashMap<String, usize>,
    mode: Mode,
    taffy: Taffy,
    root_node: Node,
}

impl Compositor {
    pub fn new(size: Rect, mode: Mode) -> Self {
        let mut taffy = Taffy::new();

        let root_node = taffy
            .new_leaf(Style {
                flex_direction: FlexDirection::Column,
                justify_content: Some(JustifyContent::FlexEnd),
                size: Size {
                    width: Dimension::Points(f32::from(size.width)),
                    height: Dimension::Points(f32::from(size.height)),
                },
                ..Default::default()
            })
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

        Self {
            active_document_idx: 0,
            documents: Vec::default(),
            document_name_indexes: HashMap::new(),
            mode,
            taffy,
            root_node,
        }
    }

    fn buffer_space(&self) -> Rect {
        self.taffy.layout(self.root_node).unwrap().into()
    }
}

impl Component for Compositor {
    fn update(&mut self, msg: Message) -> Result<Option<Command>> {
        if let Message::EnterMode(mode) = msg.clone() {
            if let Mode::Insert = mode {
                if self.documents.is_empty() {
                    self.documents
                        .push(Document::empty(self.mode.clone(), self.buffer_space()));
                }
            }

            self.mode = mode;
        }

        if let Message::VisualSplit = msg.clone() {
            let buffer_space = self.buffer_space();

            self.documents[self.active_document_idx].set_viewport(
                0,
                Rect::positioned(
                    buffer_space.width / 2,
                    buffer_space.height,
                    buffer_space.left(),
                    buffer_space.top(),
                ),
            );
            self.documents[self.active_document_idx].add_view(Rect::positioned(
                buffer_space.width / 2,
                buffer_space.height,
                buffer_space.width / 2,
                buffer_space.top(),
            ));
            self.documents[self.active_document_idx].set_active_view(1);
        }

        if let Message::PreviousWindow = msg.clone() {
            self.documents[self.active_document_idx].set_active_view(0);
        }

        if let Message::Open(path) = msg.clone() {
            if !self.document_name_indexes.contains_key(&path) {
                self.documents.push(
                    //TODO: drop this
                    Document::from(
                        path.clone().as_str(),
                        self.mode.clone(),
                        self.buffer_space(),
                        &mut io::BufReader::new(
                            File::open(path.clone().as_str()).context("opening file")?,
                        ),
                    )
                    .context("opening document")?,
                );
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

        if !self.documents.is_empty() {
            return self.documents[self.active_document_idx].update(msg);
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
            self.documents.iter().for_each(|doc| doc.render_to(frame));
        }

        if let Mode::Normal(_) | Mode::Insert = self.mode {
            frame.set_cursor_position(if self.documents.is_empty() {
                self.taffy.layout(self.root_node).unwrap().location.into()
            } else {
                let body_position: Position =
                    self.taffy.layout(self.root_node).unwrap().location.into();
                Position::new(
                    self.documents[self.active_document_idx]
                        .cursor_position()
                        .col
                        .saturating_add(body_position.col),
                    self.documents[self.active_document_idx]
                        .cursor_position()
                        .row
                        .saturating_add(body_position.row),
                )
            });
        }
    }
}
