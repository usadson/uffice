// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::ops::{
    Deref,
    DerefMut,
};

use winit::window::CursorIcon;

use super::{
    painter::Painter,
    Position,
    Rect,
};

pub mod document_view;

#[derive(Debug)]
pub enum View {
    Document(document_view::DocumentView),
}

impl View {

}

impl Deref for View {
    type Target = dyn ViewImpl;

    fn deref(&self) -> &Self::Target {
        match self {
            View::Document(view) => view
        }
    }
}

impl DerefMut for View {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            View::Document(view) => view
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewData {

}

pub trait ViewImpl {
    /// This function is used so the scroller knows how much we're able to
    /// scroll.
    fn calculate_content_height(&self) -> f32;

    fn check_interactable_for_mouse(&mut self, mouse_position: Position<f32>,
        callback: &mut dyn FnMut(&mut crate::wp::Node, Position<f32>)) -> bool;

    /// Print the document tree to stdout.
    fn dump_dom_tree(&mut self);

    fn handle_event(&mut self, event: &mut Event);
}

#[derive(Debug)]
pub enum Event<'a> {
    Paint(PaintEvent<'a>),

    MouseMoved(Position<f32>, &'a mut Option<CursorIcon>),
}

pub struct PaintEvent<'a> {
    pub content_rect: Rect<f32>,
    /// The opaqueness of the view, from 0.0 to 1.0 inclusive.
    pub opaqueness: f32,
    pub start_y: f32,
    pub painter: &'a mut dyn Painter,
    pub zoom: f32,
}

impl<'a> core::fmt::Debug for PaintEvent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaintEvent")
            .field("content_rect", &self.content_rect)
            .field("opaqueness", &self.opaqueness)
            .field("start_y", &self.start_y)
            .field("painter", &String::from("<impl>"))
            .field("zoom", &self.zoom)
            .finish()
    }
}
