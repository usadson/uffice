// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.
//
// This file contains a Win32-specific painter, a software renderer which
// targets Windows-platforms. It uses some nice abstractions to use the
// Windows APIs relating to painting, but doesn't expose them since they're
// not relevant for other systems.

use std::{rc::Rc, cell::RefCell, collections::{HashMap, hash_map::Entry}, hash::Hash};

use winit::{
    window::{
        Window,
    },
};

use raw_window_handle::HasRawWindowHandle;

use crate::gui::{
    Brush,
    Rect,
    Color, Position, Size
};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    WindowsError(windows::core::Error),

    MltgError(mltg::Error),
}

impl From<windows::core::Error> for Error {
    fn from(value: windows::core::Error) -> Self {
        Self::WindowsError(value)
    }
}

impl From<mltg::Error> for Error {
    fn from(value: mltg::Error) -> Self {
        Self::MltgError(value)
    }
}

#[derive(Debug)]
enum PaintCommand {
    Rect {
        brush: Brush,
        rect: Rect<f32>
    },
    Text {
        brush: Brush,
        position: Position<f32>,
        layout: mltg::TextLayout,
    },
}

impl From<Rect<f32>> for mltg::Rect<f32> {
    fn from(value: Rect<f32>) -> Self {
        Self::from_points(
            (value.left, value.top),
            (value.right, value.bottom)
        )
    }
}

impl From<Color> for mltg::Rgba<f32> {
    fn from(value: Color) -> Self {
        Self::new(
            value.red() as f32 / 1.0,
            value.green() as f32 / 1.0,
            value.blue() as f32 / 1.0,
            value.alpha() as f32 / 1.0
        )
    }
}

impl From<Position<f32>> for mltg::Point<f32> {
    fn from(value: Position<f32>) -> Self {
        Self::new(value.x(), value.y())
    }
}

impl From<mltg::Size<f32>> for Size<f32> {
    fn from(value: mltg::Size<f32>) -> Self {
        Self::new(value.width, value.height)
    }
}

struct SharedCacheSources {
    font_source: font_kit::sources::multi::MultiSource,
}

impl SharedCacheSources {
    pub fn new() -> Self {
        Self {
            font_source: font_kit::sources::multi::MultiSource::from_sources(crate::fonts::resolve_font_sources()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum SelectOption<T> {
    /// This option was never set.
    NeverSelected,

    /// This option was set, but is now cleared.
    Cleared,

    Some(T),
}

impl<T> SelectOption<T> {
    pub fn unwrap(self) -> T {
        match self {
            SelectOption::NeverSelected => panic!("unwrapped an option that was never selected"),
            SelectOption::Cleared => panic!("unwrapped an option that was cleared"),
            SelectOption::Some(t) => t,
        }
    }

    fn as_ref(&self) -> SelectOption<&T> {
        match *self {
            SelectOption::NeverSelected => SelectOption::NeverSelected,
            SelectOption::Cleared => SelectOption::Cleared,
            SelectOption::Some(ref t) => SelectOption::Some(t),
        }
    }
}
struct CachedFont {
    parent: Rc<RefCell<CachedFontFamily>>,
    format: mltg::TextFormat,
}

struct CachedFontFamily {
    types: HashMap<mltg::TextStyle, Rc<CachedFont>>,
}

struct Win32PainterCache {
    #[allow(dead_code)]
    sources: Rc<RefCell<SharedCacheSources>>,

    font_families: HashMap<String, Rc<RefCell<CachedFontFamily>>>,
}

fn load_font(sources: &Rc<RefCell<SharedCacheSources>>, factory: &mltg::Factory, font: super::FontSpecification) -> Result<(mltg::TextStyle, mltg::TextFormat), super::FontSelectionError> {
    let properties = font_kit::properties::Properties::new();

    // TODO bold, italic, etc.

    let family_names = [
        font_kit::family_name::FamilyName::Title(String::from(font.family_name))
    ];

    use font_kit::handle::Handle;
    use font_kit::error::SelectionError;

    match sources.as_ref().borrow().font_source.select_best_match(&family_names, &properties) {
        Ok(result) => {
            let owning_bytes;
            let owning_path;

            let d2_font = match result {
                Handle::Memory { bytes, font_index: _ } => {
                    owning_bytes = bytes;
                    mltg::Font::Memory(&owning_bytes, font.family_name)
                }
                Handle::Path { path, font_index: _ } => {
                    owning_path = path;
                    mltg::Font::File(&owning_path, font.family_name)
                }
            };

            let style = mltg::TextStyle{
                ..Default::default()
            };

            let format = factory.create_text_format(d2_font, font.size, Some(&style), None).expect("Failed to create text format");

            Ok((style, format))
        }
        Err(e) => Err(match e {
            SelectionError::CannotAccessSource => super::FontSelectionError::CannotAccessResource,
            SelectionError::NotFound => super::FontSelectionError::NotFound,
        })
    }
}

impl Win32PainterCache {
    pub fn insert_font(&mut self, font_spec: super::FontSpecification, font: (mltg::TextStyle, mltg::TextFormat)) -> Rc<CachedFont> {
        let (style, format) = font;
        match self.font_families.entry(String::from(font_spec.family_name)) {
            Entry::Occupied(o) => {
                let family = o.get().clone();
                let mut family = family.borrow_mut();
                let cached_font = Rc::new(CachedFont {
                    parent: o.get().clone(),
                    format
                });

                family.types.insert(style, cached_font.clone());
                cached_font
            }
            Entry::Vacant(v) => {
                let family = Rc::new(RefCell::new(
                    CachedFontFamily {
                        types: HashMap::new()
                    }
                ));

                let cached_font = Rc::new(CachedFont { parent: family.clone(), format });
                family.borrow_mut().types.insert(style, cached_font.clone());

                v.insert(family);

                cached_font
            }
        }
    }

    pub fn find_cached_font(&self, font: super::FontSpecification) -> Option<Rc<CachedFont>> {
        match self.font_families.get(font.family_name) {
            Some(family) => {
                Some(family.as_ref().borrow().types.values().next().unwrap().clone())
            }
            None => None
        }
    }
}

/// TODO this struct should support Drop entirely, but mltg neither supports
/// Drop nor interfacing with windows' Direct2D structs, making e.g
/// `IDXGISwapChain1->Release()` impossible.
pub struct Win32Painter {
    window_size: winit::dpi::PhysicalSize<u32>,
    window_scale_factor: f32,

    context: mltg::Context<mltg::Direct2D>,
    factory: mltg::Factory,
    render_target: mltg::d2d::RenderTarget,

    shared_cache_sources: Rc<RefCell<SharedCacheSources>>,
    caches: HashMap<super::PainterCache, Win32PainterCache>,
    current_cache: super::PainterCache,

    selected_font: SelectOption<Rc<CachedFont>>,

    commands: Vec<PaintCommand>,
}

impl Win32Painter {
    pub fn new(window: &mut Window) -> Result<Self, Error> {
        println!("Creating context");
        let context = mltg::Context::new(mltg::Direct2D::new()?)?;
        println!("Created  context");

        let factory = context.create_factory();
        println!("Created  factory");

        let window_size = window.inner_size();
        println!("Mapped   window size: {:?}", window_size);

        println!("Creating render target for window {:?}", window.raw_window_handle());
        let render_target = context.create_render_target(
            window.raw_window_handle(), (window_size.width, window_size.height)).unwrap();

        println!("Created  render target");

        let painter = Self {
            window_size: window.inner_size(),
            window_scale_factor: window.scale_factor() as _,
            context,
            factory,
            render_target,

            shared_cache_sources: Rc::new(RefCell::new(SharedCacheSources::new())),
            caches: HashMap::new(),
            current_cache: crate::gui::painter::PainterCache::UI,

            selected_font: SelectOption::NeverSelected,

            commands: Vec::new(),
        };

        Ok(painter)
    }

    /// Translate the library-agnostic gui::Brush into the Direct2D Brush of
    /// the mltg library.
    fn translate_brush(&self, brush: &Brush) -> mltg::Brush {
        match brush {
            Brush::SolidColor(color) => {
                // TODO there should be an API for changing the color of a
                // solid color brush in mltg, since Direct2D does support it.

                self.factory.create_solid_color_brush(*color).expect("Failed to translate brush")
            }
        }
    }

    /// Ensures the provided cache is allocated.
    fn ensure_cache_created(&mut self, cache: super::PainterCache) -> &mut Win32PainterCache {
        match self.caches.entry(cache) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(
                Win32PainterCache {
                    sources: self.shared_cache_sources.clone(),
                    font_families: HashMap::new(),
                }
            )
        }
    }

    /// Get the current cache and create one if absent.
    fn current_cache(&mut self) -> &mut Win32PainterCache {
        self.ensure_cache_created(self.current_cache)
    }
}

impl super::Painter for Win32Painter {
    fn clear_cache(&mut self, cache: super::PainterCache) {
        self.caches.remove(&cache);
    }

    fn display(&mut self) {
        self.context.set_scale_factor(self.window_scale_factor);

        self.context.draw(&self.render_target, |target_cmd| {
            target_cmd.clear((0.1, 0.1, 0.1, 1.0));

            for command in &self.commands {
                match command {
                    PaintCommand::Rect { brush, rect } => {
                        target_cmd.fill(&Into::<mltg::Rect<f32>>::into(*rect), &self.translate_brush(brush));
                    }
                    PaintCommand::Text { brush, position, layout } => {
                        target_cmd.fill(&layout.position(*position), &self.translate_brush(brush));
                    }
                }
            }

        }).expect("Failed to paint");
    }

    fn handle_resize(&mut self, window: &mut winit::window::Window) {
        self.window_size = window.inner_size();
        self.window_scale_factor = window.scale_factor() as _;
        self.context.resize_target(&mut self.render_target, (self.window_size.width, self.window_size.height))
            .expect("Failed to resize render target");
    }

    fn paint_rect(&mut self, brush: Brush, rect: Rect<f32>) {
        self.commands.push(PaintCommand::Rect { brush, rect })
    }

    fn paint_text(&mut self, brush: Brush, position: crate::gui::Position<f32>, text: &str) -> Size<f32> {
        let layout = self.factory.create_text_layout(text, &self.selected_font.as_ref().unwrap().format, mltg::TextAlignment::Leading, None)
            .unwrap();
        let size = layout.size();
        self.commands.push(PaintCommand::Text { brush, position, layout });
        size.into()
    }

    fn reset(&mut self) {
        self.commands.clear();
        self.current_cache = crate::gui::painter::PainterCache::UI;
    }

    fn select_font(&mut self, font_spec: super::FontSpecification) -> Result<(), super::FontSelectionError> {
        if let Some(font) = self.current_cache().find_cached_font(font_spec) {
            self.selected_font = SelectOption::Some(font);
            return Ok(());
        }

        //
        // Search in other caches to clone reference.
        //

        let mut found_font = None;
        for cache in self.caches.values() {
            if let Some(font) = cache.find_cached_font(font_spec) {
                found_font = Some(font);
                break;
            }
        }

        if let Some(font) = found_font {
            // Add a reference from the other cache into the current cache.
            self.current_cache().font_families.insert(String::from(font_spec.family_name), font.parent.clone());

            self.selected_font = SelectOption::Some(font);
            return Ok(());
        }

        //
        // Load the font, since no cache contains this font.
        //

        match load_font(&self.shared_cache_sources, &self.factory, font_spec) {
            Ok(font) => {
                self.selected_font = SelectOption::Some(self.current_cache().insert_font(font_spec, font));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn switch_cache(&mut self, cache: super::PainterCache) {
        self.current_cache = cache;
        self.ensure_cache_created(cache);
        self.selected_font = SelectOption::Cleared;
    }
}
