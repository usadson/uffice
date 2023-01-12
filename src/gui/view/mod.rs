// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::ops::{Deref, DerefMut};

use sfml::{system::{Vector2f, Vector2}, window::CursorType, graphics::RenderWindow};

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

    fn check_interactable_for_mouse(&self, mouse_position: sfml::system::Vector2<f32>,
        callback: &mut dyn FnMut(&mut crate::wp::Node, crate::text_settings::Position)) -> bool;

    /// Print the document tree to stdout.
    fn dump_dom_tree(&self);

    fn handle_event(&mut self, event: &mut Event);
}

#[derive(Debug)]
pub enum Event<'a> {
    Draw(DrawEvent<'a>),

    MouseMoved(Vector2f, &'a mut Option<CursorType>),
}

#[derive(Debug)]
pub struct DrawEvent<'a> {
    /// The opaqueness of the view, from 0.0 to 1.0 inclusive.
    pub opaqueness: f32,
    pub start_y: f32,
    pub window: &'a mut RenderWindow,
    pub window_size: Vector2<u32>,
    pub zoom: f32,
}

