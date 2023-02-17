// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{cell::RefCell, rc::Rc};
use bitflags::bitflags;
use super::{Brush, Rect, Position, Size};

#[cfg(windows)]
pub mod win32;

#[derive(Debug)]
pub enum FontSelectionError {
    /// Failed to access the resource associated with the font.
    CannotAccessResource,

    /// Failed to find the font with the specified options.
    NotFound,
}

/// https://learn.microsoft.com/en-us/typography/opentype/spec/os2#usweightclass
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub enum FontWeight {
    Custom(f32),

    Thin,
    ExtraLight,
    Light,
    SemiLight,
    #[default] Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl From<FontWeight> for f32 {
    /// Convert the FontWeight into the industry-standard numeric format.
    fn from(value: FontWeight) -> Self {
        match value {
            FontWeight::Custom(value) => value,
            FontWeight::Thin => 100.0,
            FontWeight::ExtraLight => 200.0,
            FontWeight::Light => 300.0,
            FontWeight::SemiLight => 350.0,
            FontWeight::Regular => 400.0,
            FontWeight::Medium => 500.0,
            FontWeight::SemiBold => 600.0,
            FontWeight::Bold => 700.0,
            FontWeight::ExtraBold => 800.0,
            FontWeight::Black => 900.0,
        }
    }
}

bitflags! {
    pub struct FontStyle: u32 {
        const NORMAL = 0;
        const BOLD = 1;
        const ITALIC = 2;
        const UNDERLINE = 4;
        const STRIKEOUT = 8;
    }
}

/// Specifies what font to use.
#[derive(Debug, Clone, Copy)]
pub struct FontSpecification<'a> {
    family_name: &'a str,
    size: f32,
    weight: FontWeight,
}

impl<'a> FontSpecification<'a> {
    pub fn new(family_name: &'a str, size: f32, weight: FontWeight) -> FontSpecification<'a> {
        Self {
            family_name,
            size,
            weight
        }
    }

    pub fn family_name(&self) -> &str {
        self.family_name
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn weight(&self) -> FontWeight {
        self.weight
    }
}

/// Specifies the quality of the painter. For example, when a font is currently
/// present, but it isn't generated with the correct size, the following
/// behaviors differ:
///
/// 1. For `AvoidResourceRescalingForDetail`, the most similar font is used,
///    scaling the text as if it was the correct size. This prevents loading
///    extra resources for performance, but produces a worse looking image.
///
/// 2. For `Full`, it is resized (most likely keeping the other as well).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaintQuality {
    /// Avoid rescaling resources when the rescaled versions would look better.
    AvoidResourceRescalingForDetail,

    /// Always load the best quality fonts and images.
    Full,
}

/// The PainterCache specifies which cache to use when painting. This way, we
/// can clear a certain cache without clearing too much.
///
/// This is especially useful when a document uses a lot of fonts, but the
/// other open documents and the UI don't use those fonts. When that document
/// is closed, we can clear that cache without having cache misses in the next
/// repaint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PainterCache {
    /// The user-interface cache. This is the default cache.
    UI,

    /// The cache for a certain document. ID allocation and management is not
    /// in the scope of the GUI painter API.
    Document(usize),
}

/// Calculate properties about text in order to do layout without the need of
/// claiming the Painter. This allows us to do layout in the background while
/// the main UI thread can still render and run the main loop.
pub trait TextCalculator {

    fn calculate_text_size(&mut self, font: FontSpecification, text: &str) -> Result<Size<f32>, FontSelectionError>;

    fn line_spacing(&mut self, font: FontSpecification) -> Result<f32, FontSelectionError>;

}

/// Paint on a window using specific functions. The underlying implementation
/// might schedule paint tasks, so the commands might not get processed
/// immediately.
///
/// ## Commands
/// Commands are the requested paint functions, such as [paint_rect](paint_rect).
pub trait Painter {

    /// Begins a clip region. Make sure to end this using
    /// [end_clip_region](end_clip_region).
    fn begin_clip_region(&mut self, rect: Rect<f32>);

    /// Clears a certain cache. This frees up memory for this given cache.
    fn clear_cache(&mut self, cache: PainterCache);

    /// Process the paint commands.
    ///
    /// This is only applicable for Painters that schedule the commands, other
    /// painters can ignore this function.
    fn display(&mut self);

    fn end_clip_region(&mut self);

    /// Called when the window, client rect, etc resizes.
    fn handle_resize(&mut self, window: &mut winit::window::Window);

    /// Paint a rect using the specified brush.
    fn paint_rect(&mut self, brush: Brush, rect: Rect<f32>);

    /// Paint the text using the specified brush. Returns the size of the text
    /// in pixels.
    fn paint_text(&mut self, brush: Brush, position: Position<f32>, text: &str, size: Option<Size<f32>>) -> Size<f32>;

    /// Prepare for new paint commands.
    fn reset(&mut self);

    /// Changes the current font to the specified font below, which uses
    /// caching to improve performance.
    fn select_font(&mut self, font: FontSpecification) -> Result<(), FontSelectionError>;

    /// Switches to a certain cache. When it is not created or cleared, it will
    /// be allocated for you.
    fn switch_cache(&mut self, cache: PainterCache, quality: PaintQuality);

    /// Get the sharable text calculator.
    fn text_calculator(&mut self) -> Rc<RefCell<dyn TextCalculator>>;

}
