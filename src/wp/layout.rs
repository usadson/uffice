// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use crate::{
    gui::Position,
    text_settings::PageSettings,
    word_processing::TWELFTEENTH_POINT,
};

pub struct LineLayout {
    line_height: f32,

    pub position_on_line: Position<f32>,

    pub page_horizontal_start: f32,
    pub page_horizontal_end: f32,
    pub page_vertical_start: f32,
    pub page_vertical_end: f32,
}

impl LineLayout {
    pub fn new(page_settings: &PageSettings, y: f32) -> Self {
        Self {
            line_height: 0.0,
            position_on_line: Position::new(page_settings.margins.left() as f32 * TWELFTEENTH_POINT, y),

            page_horizontal_start: page_settings.margins.left() as f32 * TWELFTEENTH_POINT,
            page_horizontal_end: (page_settings.size.width() - page_settings.margins.right()) as f32 * TWELFTEENTH_POINT,

            page_vertical_start: page_settings.margins.top() as f32 * TWELFTEENTH_POINT,
            page_vertical_end: (page_settings.size.height() - page_settings.margins.bottom()) as f32 * TWELFTEENTH_POINT,
        }
    }

    /// Adds a line-height candidate. When the supplied height is smaller than
    /// the current height, nothing will happen.
    pub fn add_line_height_candidate(&mut self, height: f32) {
        if height > self.line_height {
            self.line_height = height;
        }
    }

    pub fn new_line(&mut self) {
        let new_y = self.position_on_line.y() + self.line_height;
        self.position_on_line = Position::new(self.page_horizontal_start, new_y);
        self.line_height = 0.0;
    }

    pub fn reset(&mut self) {
        self.position_on_line = Position::new(self.page_horizontal_start, self.page_vertical_start);
        self.line_height = 0.0;
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }
}
