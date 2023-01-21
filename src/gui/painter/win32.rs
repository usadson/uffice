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
        exact_size: Option<mltg::Size<f32>>,
    },

    BeginClipRegion {
        rect: Rect<f32>,
    },
    EndClipRegion,
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
            value.red() as f32 / 255.0,
            value.green() as f32 / 255.0,
            value.blue() as f32 / 255.0,
            value.alpha() as f32 / 255.0
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

impl From<Size<f32>> for mltg::Size<f32> {
    fn from(value: Size<f32>) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<super::FontWeight> for font_kit::properties::Weight {
    fn from(value: super::FontWeight) -> Self {
        use font_kit::properties::Weight;
        match value {
            super::FontWeight::Custom(weight) => Weight(weight),

            super::FontWeight::Thin => Weight::THIN,
            super::FontWeight::ExtraLight => Weight::EXTRA_LIGHT,
            super::FontWeight::Light => Weight(350.0),
            super::FontWeight::SemiLight => Weight::LIGHT,
            super::FontWeight::Regular => Weight::NORMAL,
            super::FontWeight::Medium => Weight::MEDIUM,
            super::FontWeight::SemiBold => Weight::SEMIBOLD,
            super::FontWeight::Bold => Weight::BOLD,
            super::FontWeight::ExtraBold => Weight::EXTRA_BOLD,
            super::FontWeight::Black => Weight::BLACK,
        }
    }
}

impl From<super::FontWeight> for mltg::FontWeight {
    fn from(value: super::FontWeight) -> Self {
        match value {
            super::FontWeight::Custom(_weight) => todo!("mltg is missing this API"),

            super::FontWeight::Thin => mltg::FontWeight::Thin,
            super::FontWeight::ExtraLight => mltg::FontWeight::UltraLight,
            super::FontWeight::Light => mltg::FontWeight::Light,
            super::FontWeight::SemiLight => mltg::FontWeight::SemiLight,
            super::FontWeight::Regular => mltg::FontWeight::Regular,
            super::FontWeight::Medium => mltg::FontWeight::Medium,
            super::FontWeight::SemiBold => mltg::FontWeight::SemiBold,
            super::FontWeight::Bold => mltg::FontWeight::Bold,
            super::FontWeight::ExtraBold => mltg::FontWeight::UltraBold,
            super::FontWeight::Black => mltg::FontWeight::UltraBlack,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord)]
struct FontVariantCacheKey {
    /// Some weird conversion in order to have the Hash trait.
    size: u64,

    /// We only care about the 350.5 decimal, not the others.
    weight: u32,
}

impl<'a> From<super::FontSpecification<'a>> for FontVariantCacheKey {
    fn from(value: super::FontSpecification<'a>) -> Self {
        Self {
            size: (value.size * 10.0) as u64,
            weight: (Into::<f32>::into(value.weight) * 10.0) as u32,
        }
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
    types: HashMap<FontVariantCacheKey, Rc<CachedFont>>,
}

struct Win32PainterCache {
    #[allow(dead_code)]
    sources: Rc<RefCell<SharedCacheSources>>,

    font_families: HashMap<String, Rc<RefCell<CachedFontFamily>>>,
}

fn load_font(sources: &Rc<RefCell<SharedCacheSources>>, factory: &mltg::Factory, font: super::FontSpecification) -> Result<(mltg::TextStyle, mltg::TextFormat), super::FontSelectionError> {
    println!("[Painter(Win32)] Loading new font \"{}\" with size {}", font.family_name, Into::<FontVariantCacheKey>::into(font).size);
    let properties = font_kit::properties::Properties {
        weight: font.weight.into(),
        ..Default::default()
    };

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
                weight: font.weight.into(),
                ..Default::default()
            };

            let format = factory.create_text_format(d2_font, mltg::font_point(font.size), Some(&style), None).expect("Failed to create text format");

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
        let (_style, format) = font;
        match self.font_families.entry(String::from(font_spec.family_name)) {
            Entry::Occupied(o) => {
                let family = o.get().clone();
                let mut family = family.borrow_mut();
                let cached_font = Rc::new(CachedFont {
                    parent: o.get().clone(),
                    format
                });

                family.types.insert(font_spec.into(), cached_font.clone());
                cached_font
            }
            Entry::Vacant(v) => {
                let family = Rc::new(RefCell::new(
                    CachedFontFamily {
                        types: HashMap::new()
                    }
                ));

                let cached_font = Rc::new(CachedFont { parent: family.clone(), format });
                let previous = family.borrow_mut().types.insert(font_spec.into(), cached_font.clone());
                assert!(previous.is_none(), "Loaded a new font for nothing!");

                v.insert(family);

                cached_font
            }
        }
    }

    pub fn find_cached_font(&self, font: super::FontSpecification) -> Option<Rc<CachedFont>> {
        match self.font_families.get(font.family_name) {
            Some(family) => {
                family.as_ref().borrow().types.get(&font.into()).cloned()
            }
            None => None
        }
    }

    pub fn find_cached_font_closest(&self, font: super::FontSpecification) -> Option<Rc<CachedFont>> {
        match self.font_families.get(font.family_name) {
            Some(family) => {
                let font = Into::<FontVariantCacheKey>::into(font);

                let types = &family.as_ref().borrow().types;

                let mut closest_index = None;
                let mut closest_size_diff = None;

                for i in 0..types.len() {
                    let size = types.keys().nth(i).unwrap().size;
                    if size == font.size {
                        closest_index = Some(i);
                        break;
                    }

                    let size_diff = (size as i64 - font.size as i64).abs();

                    if closest_index.is_none() || closest_size_diff.unwrap() > size_diff{
                        closest_index = Some(i);
                        closest_size_diff = Some(size_diff);
                    }
                }

                match closest_index {
                    None => None,
                    Some(index) => {
                        Some(types.values().nth(index).unwrap().clone())
                    }
                }
            }
            None => None
        }
    }
}

pub struct Win32TextCalculator {
    factory: mltg::Factory,

    cache: Win32PainterCache,
}

impl Win32TextCalculator {
    fn new(cache: Win32PainterCache) -> Self {
        let context = mltg::Context::new(mltg::Direct2D::new().unwrap()).unwrap();
        let factory = context.create_factory();

        Self { factory, cache }
    }

    fn get_font(&mut self, font_spec: super::FontSpecification) -> Result<Rc<CachedFont>, FontSelectionError> {
        if let Some(cached_font) = self.cache.find_cached_font(font_spec) {
            return Ok(cached_font);
        }

        let loaded_font = load_font(&self.cache.sources, &self.factory, font_spec)?;
        Ok(self.cache.insert_font(font_spec, loaded_font))
    }
}

impl super::TextCalculator for Win32TextCalculator {
    fn calculate_text_size(&mut self, font_spec: super::FontSpecification, text: &str) -> Result<Size<f32>, FontSelectionError> {
        let font = self.get_font(font_spec)?;
        Ok(self.factory.create_text_layout(text, &font.format, mltg::TextAlignment::Leading, None).unwrap().size().into())
    }

    fn line_spacing(&mut self, font: super::FontSpecification) -> Result<f32, FontSelectionError> {
        Ok(self.get_font(font)?.format.line_spacing().unwrap().height)
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
    quality: super::PaintQuality,

    selected_font: SelectOption<Rc<CachedFont>>,

    commands: Vec<PaintCommand>,

    text_calculator: Option<Rc<RefCell<Win32TextCalculator>>>,
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
            quality: super::PaintQuality::Full,

            selected_font: SelectOption::NeverSelected,

            commands: Vec::new(),
            text_calculator: None,
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

    fn begin_clip_region(&mut self, rect: Rect<f32>) {
        self.commands.push(PaintCommand::BeginClipRegion { rect });
    }

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
                    PaintCommand::Text { brush, position, layout, exact_size, } => {
                        let mut position = *position;
                        if let Some(exact_size) = exact_size.clone() {
                            //target_cmd.push_clip(mltg::Rect::new(*position, exact_size));
                            let scale_x = exact_size.width / layout.size().width;
                            let scale_y = exact_size.height / layout.size().height;
                            target_cmd.scale(mltg::Size::new(scale_x, scale_y));
                            position = Position::new(
                                position.x / scale_x,
                                position.y / scale_y
                            );
                        }

                        target_cmd.fill(&layout.position(position), &self.translate_brush(brush));

                        if exact_size.is_some() {
                            target_cmd.reset_transform();
                            //target_cmd.pop_clip();
                        }
                    }
                    PaintCommand::BeginClipRegion { rect } => target_cmd.push_clip(*rect),
                    PaintCommand::EndClipRegion => target_cmd.pop_clip(),
                }
            }

        }).expect("Failed to paint");
    }

    fn end_clip_region(&mut self) {
        self.commands.push(PaintCommand::EndClipRegion);
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

    fn paint_text(&mut self, brush: Brush, position: crate::gui::Position<f32>, text: &str, size: Option<Size<f32>>) -> Size<f32> {
        let exact_size = match size {
            None => None,
            Some(size) => Some(size.into())
        };
        let layout = self.factory.create_text_layout(text, &self.selected_font.as_ref().unwrap().format, mltg::TextAlignment::Leading, None)
            .unwrap();
        let size = layout.size();
        self.commands.push(PaintCommand::Text { brush, position, layout, exact_size });
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

        if self.quality == super::PaintQuality::AvoidResourceRescalingForDetail {
            if let Some(font) = self.current_cache().find_cached_font_closest(font_spec) {
                self.selected_font = SelectOption::Some(font);
                return Ok(());
            }
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

    fn switch_cache(&mut self, cache: super::PainterCache, quality: super::PaintQuality) {
        self.quality = quality;
        self.current_cache = cache;
        self.ensure_cache_created(cache);
        self.selected_font = SelectOption::Cleared;
    }

    fn text_calculator(&mut self) -> Rc<RefCell<dyn super::TextCalculator>> {
        match self.text_calculator.as_ref() {
            Some(calculator) => calculator.clone(),
            None => {
                let calculator = Rc::new(
                    RefCell::new(
                        Win32TextCalculator::new(
                            Win32PainterCache {
                                sources: self.shared_cache_sources.clone(),
                                font_families: HashMap::new(),
                            }
                        )
                    )
                );

                self.text_calculator = Some(calculator.clone());
                calculator
            }
        }
    }
}
