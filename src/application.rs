// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use font_kit::family_name::FamilyName;
use roxmltree as xml;

use sfml::SfBox;
use sfml::graphics::*;
use sfml::system::Vector2f;
use sfml::window::*;

use notify::{Watcher, RecursiveMode};

use uffice_lib::{profile_expr, profiling::Profiler, math};

use crate::relationships::Relationships;
use crate::style::StyleManager;
use crate::text_settings::Position;
use crate::word_processing;
use crate::word_processing::DocumentResult;
use crate::wp;
use crate::wp::MouseEvent;
use crate::wp::numbering::NumberingManager;

pub const SCROLL_BAR_WIDTH: f32 = 20.0;

/// The color of the scroll bar below the scroll thumb.
const SCROLL_BAR_BACKGROUND_COLOR: Color = Color::rgb(0xBD, 0xBD, 0xBD);

/// The color of the thumb of the scrollbar when it's neither hovered nor
/// clicked.
const SCROLL_BAR_THUMB_DEFAULT_COLOR: Color = Color::rgb(0x67, 0x3A, 0xB7);

/// The color of the thumb of the scrollbar when it's hovered over.
const SCROLL_BAR_THUMB_HOVER_COLOR: Color = Color::rgb(0x65, 0x32, 0xBC);

/// The color of the thumb of the scrollbar when it's being clicked on.
const SCROLL_BAR_THUMB_CLICK_COLOR: Color = Color::rgb(0x60, 0x2B, 0xBC);

/// The height from the top to the first page, and from the end of the
/// last page to the bottom.
const VERTICAL_PAGE_MARGIN: f32 = 20.0;

/// The gaps between the pages.
const VERTICAL_PAGE_GAP: f32 = 30.0;

/// The background color of the application. This is the color under the pages.
const APPLICATION_BACKGROUND_COLOR: Color = Color::rgb(29, 28, 33);

/// The zoom levels the user can step through using control + or control -.
const ZOOM_LEVELS: [f32; 19] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.67, 0.8, 0.9, 1.0, 1.1, 1.2, 1.33, 1.5, 1.7, 2.0, 2.5, 3.0, 4.0, 5.0];

const DEFAULT_ZOOM_LEVEL_INDEX: usize = 4;

/// After how much time should a tooltip be shown (if applicable).
///
/// The following is used as a recommendation:
///     https://ux.stackexchange.com/a/360
const TOOLTIP_TIMEOUT: Duration = Duration::from_millis(500);

const TOOLTIP_BACKGROUND_COLOR: Color = Color::rgb(211, 211, 211);
const TOOLTIP_BORDER_COLOR: Color = Color::rgb(168, 168, 168);

pub fn load_archive_file_to_string(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<Rc<String>> {
    match archive.by_name(name) {
        Ok(zip_document) => Some(Rc::new(std::io::read_to_string(zip_document)
                .expect("Failed to read"))),
        Err(e) => {
            println!("Error: {} for name \"{}\"", e, name);
            None
        }
    }
}

// A4: 210 × 297
fn draw_document(archive_path: &str) -> DocumentResult {
    let mut profiler = Profiler::new(String::from("Document Rendering"));

    let archive_file = profile_expr!(profiler, "Open Archive", std::fs::File::open(archive_path)
            .expect("Failed to open specified file"));

    let mut archive = profile_expr!(profiler, "Read Archive", zip::ZipArchive::new(archive_file)
            .expect("Failed to read ZIP archive"));

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("[Document] ZIP: File: {}", file.name());
    }

    let document_relationships;
    {
        let _frame = profiler.frame(String::from("Document Relationships"));

        let txt = load_archive_file_to_string(&mut archive, "word/_rels/document.xml.rels")
                .expect("Document.xml.rels missing, assuming this is not a DOCX file.");
        if let Ok(document) = xml::Document::parse(&txt) {
            document_relationships = Relationships::load_xml(&document, &mut archive).unwrap();
        } else {
            println!("[Relationships] (word/_rels/document.xml.rels) Error!");
            document_relationships = Relationships::empty();
        }
    }

    let numbering_manager = {
        let _frame = profiler.frame(String::from("Numbering Definitions"));

        if let Some(numbering_document_text) = load_archive_file_to_string(&mut archive, "word/numbering.xml") {
            let numbering_document = xml::Document::parse(&numbering_document_text)
                .expect("Failed to parse numbering document");
            NumberingManager::from_xml(&numbering_document)
        } else {
            NumberingManager::new()
        }
    };

    let style_manager = {
        let _frame = profiler.frame(String::from("Style Definitions"));

        let styles_document_text = load_archive_file_to_string(&mut archive, "word/styles.xml")
                .expect("Style definitions missing, assuming this is not a DOCX file.");
        let styles_document = xml::Document::parse(&styles_document_text)
                .expect("Failed to parse styles document");
        StyleManager::from_document(&styles_document, &numbering_manager).unwrap()
    };

    let mut document_properties = wp::document_properties::DocumentProperties::new();
    if let Some(txt) = load_archive_file_to_string(&mut archive, "docProps/core.xml") {
        if let Ok(document) = xml::Document::parse(&txt) {
            document_properties.import_core_file_properties_part(&document);
        }
    }

    let _frame = profiler.frame(String::from("Document"));
    let document_text = load_archive_file_to_string(&mut archive, "word/document.xml")
            .expect("Archive missing word/document.xml: this file is not a WordprocessingML document!");
    let document = xml::Document::parse(&document_text)
            .expect("Failed to parse document");
    word_processing::process_document(&document, &style_manager, &document_relationships, numbering_manager, document_properties)
}

struct Scroller {
    value: f32,
    content_height: f32,
    window_height: f32,

    bar_rect: Rect<f32>,
    thumb_rect: Rect<f32>,

    is_hovered: bool,
    is_pressed: bool,

    animator: Animator,
    value_increase: f32,
}

impl Scroller {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            content_height: 0.0,
            window_height: 0.0,
            bar_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            thumb_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            is_hovered: false,
            is_pressed: false,
            animator: Animator::new_with_delay(150.0),
            value_increase: 0.0,
        }
    }

    pub fn scroll(&mut self, value: f32) {
        self.increase_thumb_position(-value / 100.0);
    }

    pub fn draw(&mut self, shape: &mut RectangleShape, parent: &mut RenderWindow) {
        let window_size = parent.size();
        self.window_height = window_size.y as f32;

        let full_page_scrolls = self.content_height / window_size.y as f32;
        let scroll_bar_height = (window_size.y as f32 / full_page_scrolls).ceil();
        let scroll_y = (window_size.y as f32 - scroll_bar_height) * Scroller::bound_position(self.value + self.value_increase);

        shape.set_fill_color(SCROLL_BAR_BACKGROUND_COLOR);
        shape.set_size(Vector2f::new(SCROLL_BAR_WIDTH, window_size.y as f32));
        shape.set_position(Vector2f::new(window_size.x as f32 - SCROLL_BAR_WIDTH, 0.0));
        self.bar_rect = shape.global_bounds();
        parent.draw(shape);

        shape.set_fill_color({
            if self.is_pressed {
                SCROLL_BAR_THUMB_CLICK_COLOR
            } else if self.is_hovered {
                SCROLL_BAR_THUMB_HOVER_COLOR
            } else {
                SCROLL_BAR_THUMB_DEFAULT_COLOR
            }
        });
        shape.set_size(Vector2f::new(SCROLL_BAR_WIDTH, scroll_bar_height));
        shape.set_position(Vector2f::new(window_size.x as f32 - SCROLL_BAR_WIDTH, scroll_y));
        self.thumb_rect = shape.global_bounds();
        parent.draw(shape);
    }

    pub fn apply_mouse_offset(&mut self, value: f32) {
        self.increase_thumb_position(value / (self.window_height as f32 - self.thumb_rect.height));
    }

    pub fn increase_thumb_position(&mut self, value: f32) {
        let increase = self.animator.update() * self.value_increase;
        self.set_thumb_position(self.value + increase);
        self.animator.reset();
        self.value_increase += value - increase;
    }

    fn set_thumb_position(&mut self, value: f32) {
        self.value = Scroller::bound_position(value);
    }

    pub fn position(&mut self) -> f32 {
        Scroller::bound_position(self.value + math::lerp_precise_f32(0.0, self.value_increase, self.animator.update()))
    }

    pub fn bound_position(value: f32) -> f32 {
        match value {
            d if d < 0.0 => 0.0,
            d if d > 1.0 => 1.0,
            d => d,
        }
    }
}

struct Animator {
    begin: Instant,
    delay_ms: f32,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            begin: Instant::now(),
            delay_ms: 220.0,
        }
    }

    pub fn new_with_delay(delay_ms: f32) -> Self {
        Self {
            begin: Instant::now(),
            delay_ms,
        }
    }

    pub fn reset(&mut self) {
        self.begin = Instant::now();
    }

    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let diff = now.duration_since(self.begin);

        if diff.as_millis() > self.delay_ms as u128 {
            return 1.0;
        }

        let value = diff.as_millis() as f32 / self.delay_ms;

        return if value > 1.0 {
            1.0
        } else {
            value
        }
    }
}

enum TooltipState {
    /// The mouse was moved but the timeout didn't expire yet.
    Unchecked,

    /// The tooltip is visible.
    Visible,

    /// The mouse hasn't moved after the timeout but there is no text to
    /// display.
    NotApplicable,
}

pub struct Application<'a> {
    archive_path: String,

    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,

    window: RenderWindow,
    cursor: SfBox<Cursor>,

    interface_font: SfBox<Font>,

    is_draw_invalidated: Arc<AtomicBool>,
    scroller: Scroller,

    scale: f32,
    document_rect: Rect<f32>,
    document: Option<Rc<RefCell<wp::Node>>>,

    zoom_index: usize,

    /// This defines how zoomed in or out the pages are.
    zoom_level: f32,

    page_textures: Vec<Rc<RefCell<RenderTexture>>>,

    mouse_position: Vector2f,
    last_mouse_move: Instant,
    tooltip_state: TooltipState,
    tooltip_text: String,

    rectangle_shape: RectangleShape<'a>,
}

fn load_interface_font() -> Option<SfBox<Font>> {
    let font_source = font_kit::source::SystemSource::new();
    let font_handle = font_source.select_best_match(&[
        FamilyName::Title(String::from("Segoe UI")),
        FamilyName::Title(String::from("Noto Sans")),
        FamilyName::SansSerif
    ], &font_kit::properties::Properties::new())
        .expect("Failed to find a system font!");

    match &font_handle {
        font_kit::handle::Handle::Memory { bytes, font_index: _ } => unsafe {
            Font::from_memory(&bytes.as_ref())
        }
        font_kit::handle::Handle::Path { path, font_index: _ } => {
            Font::from_file(path.to_str().unwrap())
        }
    }
}

impl<'a> Application<'a> {
    pub fn new(archive_path: String) -> Self {
        let is_draw_invalidated = Arc::new(AtomicBool::new(true));
        let notify_flag = is_draw_invalidated.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    println!("[Watcher] Event: {:?}", event);
                    notify_flag.store(true, Ordering::Relaxed);
                }
                Err(e) => println!("[Watcher] Failed to watch: {:?}", e),
            }
        }).expect("Failed to instantiate file watcher");

        let document_file_path = std::path::Path::new(&archive_path);
        watcher.watch(document_file_path, RecursiveMode::NonRecursive).unwrap();

        let context_settings = ContextSettings::default();
        let mut window = RenderWindow::new(VideoMode::new(1280, 720, 32),
                &format!("{} - {}", uffice_lib::constants::vendor::NAME, document_file_path.file_name().unwrap().to_string_lossy()), Style::DEFAULT, &context_settings);

        window.set_framerate_limit(30);
        window.set_active(true);

        Application {
            archive_path: archive_path.clone(),
            watcher,
            window,
            cursor: Cursor::from_system(CursorType::Arrow).unwrap(),

            interface_font: load_interface_font().unwrap(),

            is_draw_invalidated,
            scroller: Scroller::new(),
            scale: 0.0,
            document_rect: sfml::graphics::Rect::<f32>::new(0.0, 0.0, 0.0, 0.0),
            document: None,

            zoom_index: DEFAULT_ZOOM_LEVEL_INDEX,
            zoom_level: ZOOM_LEVELS[DEFAULT_ZOOM_LEVEL_INDEX],
            page_textures: Vec::new(),

            mouse_position: Vector2f::new(0.0, 0.0),
            last_mouse_move: Instant::now(),
            tooltip_state: TooltipState::NotApplicable,
            tooltip_text: String::new(),

            rectangle_shape: RectangleShape::new(),
        }
    }

    /// Calculates the height of all the pages, plus some vertical margins and
    /// gaps between the pages.
    ///
    /// This function is used so the scroller knows how much we're able to
    /// scroll.
    fn calculate_content_height(&self) -> f32 {
        // Top and bottom gaps above and below the pages.
        let mut height = VERTICAL_PAGE_MARGIN * 2.0;

        // The gaps between the pages.
        height += (self.page_textures.len() - 1) as f32 * VERTICAL_PAGE_GAP;

        for page_tex in &self.page_textures {
            height += page_tex.as_ref().borrow().size().y as f32;
        }

        height
    }

    pub fn check_interactable_for_mouse(&mut self, mouse_position: Vector2f, callback: &mut dyn FnMut(&mut wp::Node, Position))
            -> bool {
        if !self.document_rect.contains(mouse_position) {
            return false;
        }

        let mouse_position = Position::new(
            ((mouse_position.x - self.document_rect.left) / self.scale) as u32,
            ((mouse_position.y - self.document_rect.top) / self.scale) as u32
        );

        let doc = self.document.as_ref().unwrap();
        let mut document = doc.borrow_mut();

        document.hit_test(mouse_position, &mut |node| {
            callback(node, mouse_position);
        })
    }

    fn display_loading_screen(&mut self) {
        self.window.clear(APPLICATION_BACKGROUND_COLOR);

        let mut text = sfml::graphics::Text::new(&format!("Loading {}", &self.archive_path), &self.interface_font, 36);
        text.set_position((
            (self.window.size().x as f32 - text.local_bounds().width) / 2.0,
            (self.window.size().y as f32 - text.local_bounds().height) / 2.0
        ));

        self.window.draw(&text);
        self.window.display();
    }

    pub fn run(&mut self) {
        let mut shape = sfml::graphics::RectangleShape::new();
        let mut mouse_down = false;
        let mut mouse_position = Vector2f::new(0.0, 0.0);

        let mut current_cursor_type = CursorType::Arrow;
        let mut new_cursor = None;

        let mut page_introduction_animator = Animator::new();

        while self.window.is_open() {
            let window_size = self.window.size();
            {
                while let Some(event) = self.window.poll_event() {
                    match event {
                        Event::Closed => self.window.close(),
                        Event::Resized { width, height } => {
                            //self.is_draw_invalidated.store(true, Ordering::Relaxed);

                            self.window.set_view(View::new(
                                Vector2f::new(
                                    width as f32 / 2.0,
                                    height as f32 / 2.0
                                ),
                                Vector2f::new(
                                    width as f32,
                                    height as f32
                                )
                            ).deref());
                        }
                        Event::KeyPressed { code, alt, ctrl, shift, system } =>
                            self.on_key_pressed(code, alt, ctrl, shift, system),
                        Event::KeyReleased { code, alt, ctrl, shift, system } =>
                            self.on_key_released(code, alt, ctrl, shift, system),
                        Event::MouseWheelScrolled { wheel, delta, x: _, y: _ } => {
                            if wheel == sfml::window::mouse::Wheel::VerticalWheel {
                                self.scroller.scroll(delta);
                            }
                        }
                        Event::MouseButtonPressed { button, x, y } => {
                            if button == sfml::window::mouse::Button::Left {
                                mouse_position = Vector2f::new(x as f32, y as f32);

                                println!("[ClickEvent] @ {} x {}", x, y);

                                self.scroller.is_hovered = self.scroller.bar_rect.contains(mouse_position);
                                if self.scroller.is_hovered {
                                    self.scroller.is_pressed = true;
                                }

                                if !mouse_down {
                                    mouse_down = true;
                                    println!("[ClickEvent]   Document Rect @ {} x {}  w {}  h{}", self.document_rect.left, self.document_rect.top,
                                            self.document_rect.width, self.document_rect.height);
                                    self.check_interactable_for_mouse(mouse_position, &mut |node, mouse_position| {
                                        node.on_event(&mut wp::Event::Click(MouseEvent::new(mouse_position)));
                                    });
                                }
                            }
                        }
                        Event::MouseButtonReleased { button, x: _, y: _ } => {
                            if button == sfml::window::mouse::Button::Left {
                                mouse_down = false;
                                self.scroller.is_pressed = false;
                            }
                        }
                        Event::MouseMoved { x, y } => {
                            self.scroller.is_hovered = self.scroller.bar_rect.contains(mouse_position);

                            if self.scroller.is_pressed {
                                self.scroller.apply_mouse_offset(y as f32 - mouse_position.y);
                            }

                            mouse_position = Vector2f::new(x as f32, y as f32);
                            new_cursor = Some(CursorType::Arrow);

                            self.reset_tooltip(mouse_position);

                            if let Some(document) = &mut self.document {
                                document.borrow_mut().apply_recursively(&mut |node, _depth| {
                                    node.interaction_states.hover = wp::HoverState::NotHoveringOn;
                                }, 0);

                                self.check_interactable_for_mouse(mouse_position, &mut |node, position| {
                                    node.interaction_states.hover = wp::HoverState::HoveringOver;

                                    let mut event = wp::Event::Hover(wp::MouseEvent::new(position));
                                    node.on_event(&mut event);

                                    if let wp::Event::Hover(mouse_event) = &event {
                                        if let Some(cursor) = mouse_event.new_cursor {
                                            new_cursor = Some(cursor);
                                        }
                                    }
                                });
                            }
                        }
                        _ => (),
                    }
                }
            }

            if let Some(cursor) = new_cursor {
                if cursor != current_cursor_type {
                    current_cursor_type = cursor;
                    self.cursor = Cursor::from_system(cursor).unwrap();
                    unsafe {
                        self.window.set_mouse_cursor(&self.cursor);
                    }
                }
            }

            new_cursor = None;

            let factor = 0.6;
            if self.is_draw_invalidated.swap(false, Ordering::Relaxed) {
                self.display_loading_screen();

                let (new_page_textures, document) = draw_document(&self.archive_path);
                self.page_textures = new_page_textures.render_targets;
                self.document = Some(document);

                self.scroller.content_height = self.calculate_content_height() * factor;
                page_introduction_animator.reset();
            }

            self.window.clear(APPLICATION_BACKGROUND_COLOR);
//          let mut y = VERTICAL_PAGE_MARGIN * factor - self.scroller.content_height * self.scroller.position();
            let mut y = (VERTICAL_PAGE_MARGIN - self.scroller.content_height * self.scroller.value) * self.zoom_level;

            for render_texture in &self.page_textures {
                // I don't know rust well enough to be able to keep a Sprite
                // around _and_ replace the texture.
                //
                // But since this code is not performance-critical I don't care
                // atm.

                let texture = render_texture.borrow();
                let mut sprite = Sprite::with_texture(texture.texture());

                let full_size = window_size.x as f32;
                let page_size = sprite.texture_rect().width as f32;

                self.scale = full_size * self.zoom_level / page_size;
                let centered_x = (full_size - page_size * self.scale) / 2.0;
                sprite.set_scale((self.scale, self.scale));

                sprite.set_position((
                    centered_x,
                    y
                ));

                sprite.set_color(Color::rgba(255, 255, 255, (255.0 * page_introduction_animator.update()) as u8));

                y += sprite.global_bounds().size().y + VERTICAL_PAGE_GAP * self.zoom_level;

                self.document_rect = sprite.global_bounds();
                self.window.draw(&sprite);
            }

            self.draw_tooltip();

            // Scrollbar
            self.scroller.draw(&mut shape, &mut self.window);

            self.window.display();
        }

        self.window.close();
    }

    fn on_key_pressed(&self, _code: Key, _alt: bool, _ctrl: bool, _shift: bool, _system: bool) {

    }

    fn on_key_released(&mut self, code: Key, _alt: bool, ctrl: bool, _shift: bool, _system: bool) {
        // Control +
        if ctrl && code == Key::Equal {
            self.increase_zoom_level();
        }

        // Control -
        if ctrl && code == Key::Hyphen {
            self.decrease_zoom_level();
        }

        match code {
            Key::F2 => self.dump_dom_tree(),

            _ => ()
        }
    }

    fn dump_dom_tree(&self) {
        match &self.document {
            Some(document) => {
                document.borrow_mut().apply_recursively(&|node, depth| {
                    print!("🌲: {}{:?}", "    ".repeat(depth), node.data);
                    print!(" @ ({}, {})", node.position.x, node.position.y,);
                    print!(" sized ({}x{})", node.size.x, node.size.y);

                    println!();
                }, 0);
            }
            None => println!("🌲: No tree"),
        }
    }

    fn increase_zoom_level(&mut self) {
        let next_zoom_index = self.zoom_index + 1;
        if next_zoom_index < ZOOM_LEVELS.len() {
            self.zoom_index = next_zoom_index;
            self.zoom_level = ZOOM_LEVELS[next_zoom_index];
        }
    }

    fn decrease_zoom_level(&mut self) {
        if self.zoom_index != 0 {
            let next_zoom_index = self.zoom_index - 1;
            self.zoom_index = next_zoom_index;
            self.zoom_level = ZOOM_LEVELS[next_zoom_index];
        }
    }

    fn draw_tooltip(&mut self) {
        let now = Instant::now();

        match self.tooltip_state {
            TooltipState::Unchecked => {
                if now.duration_since(self.last_mouse_move) > TOOLTIP_TIMEOUT {
                    let mut tooltip_text = None;
                    let was_hit = self.check_interactable_for_mouse(self.mouse_position, &mut |node, _mouse_position| {
                        match &node.data {
                            wp::NodeData::Hyperlink(link) => {
                                if let Some(url) = link.get_url() {
                                    tooltip_text = Some(url);
                                }
                            }
                            _ => ()
                        }
                    });

                    if !was_hit || tooltip_text.is_none() {
                        self.tooltip_state = TooltipState::NotApplicable;
                        println!("Didn't hit anything @ {:?}", self.mouse_position);
                        return;
                    }

                    self.tooltip_state = TooltipState::Visible;
                    self.tooltip_text = tooltip_text.unwrap();
                }
            }
            TooltipState::NotApplicable => return,
            TooltipState::Visible => ()
        }

        if self.tooltip_text.is_empty() {
            return;
        }

        let mut text = Text::new(&self.tooltip_text, &self.interface_font, 18);
        let text_size = text.global_bounds().size();

        const TOOLTIP_PADDING: f32 = 2.0;
        const TOOLTIP_BORDER: f32 = 2.0;

        self.rectangle_shape.set_size(Vector2f::new(
            text_size.x + TOOLTIP_PADDING * 2.0,
            text_size.y + TOOLTIP_PADDING * 2.0
        ));

        self.rectangle_shape.set_outline_thickness(TOOLTIP_BORDER);
        self.rectangle_shape.set_outline_color(TOOLTIP_BORDER_COLOR);
        self.rectangle_shape.set_fill_color(TOOLTIP_BACKGROUND_COLOR);

        let rectangle_size = self.rectangle_shape.global_bounds().size();
        self.rectangle_shape.set_position(Vector2f::new(
            self.mouse_position.x,
            self.mouse_position.y - rectangle_size.y
        ));

        self.window.draw(&self.rectangle_shape);

        text.set_fill_color(Color::BLACK);

        text.set_position(Vector2f::new(
            self.rectangle_shape.position().x + TOOLTIP_PADDING,
            self.rectangle_shape.position().y,
        ));

        self.window.draw(&text);
    }

    fn reset_tooltip(&mut self, mouse_position: Vector2f) {
        self.mouse_position = mouse_position;
        self.tooltip_state = TooltipState::Unchecked;
        self.last_mouse_move = Instant::now();
        self.tooltip_text = String::new();
    }
}
