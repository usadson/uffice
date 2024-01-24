// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.
//
// This file contains a Win32-specific painter, a software renderer which
// targets Windows-platforms. It uses some nice abstractions to use the
// Windows APIs relating to painting, but doesn't expose them since they're
// not relevant for other systems.

use std::{rc::Rc, cell::RefCell, collections::{HashMap, hash_map::Entry}, hash::Hash};

use winit::window::Window;

use raw_window_handle::HasRawWindowHandle;

use crate::gui::{
    Brush,
    Rect,
    Color, Position, Size
};

use super::FontSelectionError;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
}

pub struct MacOSPainter {
    window_size: winit::dpi::PhysicalSize<u32>,
    window_scale_factor: f32,

    quality: super::PaintQuality,
}

impl MacOSPainter {
    pub fn new(window: &mut Window) -> Result<Self, Error> {
        let window_size = window.inner_size();

        let painter = Self {
            window_size: window.inner_size(),
            window_scale_factor: window.scale_factor() as _,
            quality: super::PaintQuality::Full,
        };

        Ok(painter)
    }
}

impl super::Painter for MacOSPainter {

    fn begin_clip_region(&mut self, rect: Rect<f32>) {
        todo!();
    }

    fn clear_cache(&mut self, cache: super::PainterCache) {
        todo!();
    }

    fn display(&mut self) {
        todo!();
    }

    fn end_clip_region(&mut self) {
        todo!();
    }

    fn handle_resize(&mut self, window: &mut winit::window::Window) {
        self.window_size = window.inner_size();
        self.window_scale_factor = window.scale_factor() as _;
    }

    fn paint_rect(&mut self, brush: Brush, rect: Rect<f32>) {
        todo!();
    }

    fn paint_text(&mut self, brush: Brush, position: crate::gui::Position<f32>, text: &str, size: Option<Size<f32>>) -> Size<f32> {
        todo!();
    }

    fn reset(&mut self) {

    }

    fn select_font(&mut self, font_spec: super::FontSpecification) -> Result<(), super::FontSelectionError> {
        todo!();
    }

    fn switch_cache(&mut self, cache: super::PainterCache, quality: super::PaintQuality) {
        self.quality = quality;
    }

    fn text_calculator(&mut self) -> Rc<RefCell<dyn super::TextCalculator>> {
        todo!();
    }
}
