// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use roxmltree as xml;

use sfml::SfBox;
use sfml::graphics::*;
use sfml::system::Vector2f;
use sfml::window::*;

use notify::{Watcher, RecursiveMode};

use uffice_lib::{profile_expr, profiling::{Profiler}};

use crate::relationships::Relationships;
use crate::style::StyleManager;
use crate::text_settings::Position;
use crate::word_processing;
use crate::word_processing::DocumentResult;
use crate::wp;
use crate::wp::MouseEvent;
use crate::wp::numbering::NumberingManager;

pub const SCROLL_BAR_WIDTH: f32 = 20.0;

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

    let _frame = profiler.frame(String::from("Document"));
    let document_text = load_archive_file_to_string(&mut archive, "word/document.xml")
            .expect("Archive missing word/document.xml: this file is not a WordprocessingML document!");
    let document = xml::Document::parse(&document_text)
            .expect("Failed to parse document");
    word_processing::process_document(&document, &style_manager, &document_relationships, numbering_manager)
}

struct Scroller {
    value: f32,
    document_height: f32,
    window_height: f32,

    bar_rect: Rect<f32>,
    scroll_bar_rect: Rect<f32>,
}

impl Scroller {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            document_height: 0.0,
            window_height: 0.0,
            bar_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            scroll_bar_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn scroll(&mut self, value: f32) {
        self.value -= value / 10.0;

        self.value = match self.value {
            d if d < 0.0 => 0.0,
            d if d > 1.0 => 1.0,
            d => d,
        }
    }

    pub fn offset(&self, value: f32) -> f32 {
        self.value / 5.0 * value
    }

    pub fn draw(&mut self, shape: &mut RectangleShape, parent: &mut RenderWindow) {
        let window_size = parent.size();
        self.window_height = window_size.y as f32;

        let full_page_scrolls = self.document_height / window_size.y as f32;
        let scroll_bar_height = (window_size.y as f32 / full_page_scrolls).ceil();
        let scroll_y = (window_size.y as f32 - scroll_bar_height) * self.value;

        shape.set_fill_color(Color::rgb(0xBD, 0xBD, 0xBD));
        shape.set_size(Vector2f::new(SCROLL_BAR_WIDTH, window_size.y as f32));
        shape.set_position(Vector2f::new(window_size.x as f32 - SCROLL_BAR_WIDTH, 0.0));
        self.bar_rect = shape.global_bounds();
        parent.draw(shape);

        shape.set_fill_color(Color::rgb(0x67, 0x3A, 0xB7));
        shape.set_size(Vector2f::new(SCROLL_BAR_WIDTH, scroll_bar_height));
        shape.set_position(Vector2f::new(window_size.x as f32 - SCROLL_BAR_WIDTH, scroll_y));
        self.scroll_bar_rect = shape.global_bounds();
        parent.draw(shape);
    }

    pub fn apply_mouse_offset(&mut self, value: f32) {
        self.value += value / (self.window_height as f32 - self.scroll_bar_rect.height);
    }
}

pub struct Application {
    archive_path: String,

    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,

    window: RenderWindow,
    cursor: SfBox<Cursor>,

    is_draw_invalidated: Arc<AtomicBool>,
    scroller: Scroller,

    scale: f32,
    document_rect: Rect<f32>,
    document: Option<Rc<RefCell<wp::Node>>>,
}

impl Application {
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
            is_draw_invalidated,
            scroller: Scroller::new(),
            scale: 0.0,
            document_rect: sfml::graphics::Rect::<f32>::new(0.0, 0.0, 0.0, 0.0),
            document: None,
        }
    }

    pub fn check_interactable_for_mouse(&mut self, mouse_position: Vector2f, callback: &mut dyn FnMut(&mut wp::Node, Position)) {
        if !self.document_rect.contains(mouse_position) {
            return;
        }

        let mouse_position = Position::new(
            ((mouse_position.x - self.document_rect.left) / self.scale) as u32,
            ((mouse_position.y - self.document_rect.top) / self.scale) as u32
        );

        let doc = self.document.as_ref().unwrap();
        let mut document = doc.borrow_mut();

        document.hit_test(mouse_position, &mut |node| {
            callback(node, mouse_position);
        });
    }

    pub fn run(&mut self) {
        let mut texture = sfml::graphics::RenderTexture::new(1, 1).unwrap();

        let mut shape = sfml::graphics::RectangleShape::new();
        let mut left_button_pressed = false;
        let mut mouse_down = false;
        let mut mouse_position = Vector2f::new(0.0, 0.0);

        let mut current_cursor_type = CursorType::Arrow;
        let mut new_cursor = None;

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
                        Event::MouseWheelScrolled { wheel, delta, x: _, y: _ } => {
                            if wheel == sfml::window::mouse::Wheel::VerticalWheel {
                                self.scroller.scroll(delta);
                            }
                        }
                        Event::MouseButtonPressed { button, x, y } => {
                            if button == sfml::window::mouse::Button::Left {
                                mouse_position = Vector2f::new(x as f32, y as f32);

                                println!("[ClickEvent] @ {} x {}", x, y);

                                if self.scroller.bar_rect.contains(mouse_position) {
                                    left_button_pressed = true;
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
                                left_button_pressed = false;
                                mouse_down = false;
                            }
                        }
                        Event::MouseMoved { x, y } => {
                            if left_button_pressed {
                                self.scroller.apply_mouse_offset(y as f32 - mouse_position.y);
                            }

                            mouse_position = Vector2f::new(x as f32, y as f32);
                            new_cursor = Some(CursorType::Arrow);

                            if let Some(document) = &mut self.document {
                                document.borrow_mut().apply_recursively(&mut |node| {
                                    node.interaction_states.hover = wp::HoverState::NotHoveringOn;
                                });

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

            if self.is_draw_invalidated.swap(false, Ordering::Relaxed) {
                let (new_texture, document) = draw_document(&self.archive_path);
                texture = new_texture;
                self.document = Some(document);
            }

            self.window.clear(Color::BLACK);
            {
                // I don't know rust well enough to be able to keep a Sprite
                // around _and_ replace the texture.
                //
                // But since this code is not performance-critical I don't care
                // atm.

                let mut sprite = Sprite::with_texture(texture.texture());

                let full_size = window_size.x as f32;
                let page_size = sprite.texture_rect().width as f32;
                let factor = 1.0 / 5.0 * 4.0;

                self.scale = full_size * factor / page_size;
                let centered_x = (full_size - page_size * self.scale) / 2.0;
                sprite.set_scale((self.scale, self.scale));

                sprite.set_position((
                    centered_x,
                    20.0f32 - self.scroller.offset(sprite.texture_rect().height as f32)
                ));

                self.scroller.document_height = sprite.global_bounds().height;

                self.document_rect = sprite.global_bounds();
                self.window.draw(&sprite);
            }

            // Scrollbar
            self.scroller.draw(&mut shape, &mut self.window);

            self.window.display();
        }

        self.window.close();
    }
}
