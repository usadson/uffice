// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

mod color_parser;
mod error;
mod style;
mod text_settings;
mod word_processing;

use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use roxmltree as xml;

use sfml::graphics::*;
use sfml::system::Vector2f;
use sfml::window::*;

//use font_kit;
use notify::{Watcher, RecursiveMode};
use structopt::StructOpt;
use style::StyleManager;
use text_settings::TextSettings;

pub const WORD_PROCESSING_XML_NAMESPACE: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
pub const SCROLL_BAR_WIDTH: f32 = 20.0;

fn apply_run_properties_for_paragraph_mark(element: &xml::Node, text_settings: &mut TextSettings) {
    assert_eq!(element.tag_name().name(), "rPr");

    for run_property in element.children() {
        println!("│  │  │  ├─ {}", run_property.tag_name().name());
        for attr in run_property.attributes() {
            println!("│  │  │  │  ├─ Attribute \"{}\" => \"{}\"", attr.name(), attr.value());
        }

        match run_property.tag_name().name() {
            "b" => {
                text_settings.bold = match text_settings.bold {
                    None => Some(true),
                    Some(bold) => Some(!bold)
                };
            }
            "color" => {
                for attr in run_property.attributes() {
                    println!("│  │  │  │  ├─ Color Attribute: {} => {}", attr.name(), attr.value());
                    if attr.name() == "val" {
                        text_settings.color = Some(color_parser::parse_color(attr.value()).unwrap());
                    }
                }
            }
            "rFonts" => {
                for attr in run_property.attributes() {
                    println!("│  │  │  │  ├─ Font Attribute: {} => {}", attr.name(), attr.value());
                    if attr.name() == "ascii" {
                        text_settings.font = Some(String::from(attr.value()));
                    }
                }
            }
            "sz" => {
                for attr in run_property.attributes() {
                    println!("│  │  │  │  ├─ Size Attribute: {} => {}", attr.name(), attr.value());
                    if attr.name() == "val" {
                        let new_value = str::parse::<u32>(attr.value()).expect("Failed to parse attribute");
                        println!("│  │  │  │  ├─ Value Attribute: old={:?} new={}", text_settings.non_complex_text_size, new_value);
                        text_settings.non_complex_text_size = Some(new_value);
                    }
                }
            }
            _ => ()
        }
    }
}

fn load_archive_file_to_string(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Rc<String> {
    let zip_document = archive.by_name(name).expect("Not a DOCX file");
    Rc::new(std::io::read_to_string(zip_document)
            .expect("Failed to read"))
}

// A4: 210 × 297
fn draw_document(archive_path: &str) -> RenderTexture {
    let archive_file = std::fs::File::open(archive_path)
            .expect("Failed to open specified file");

    let mut archive = zip::ZipArchive::new(archive_file)
            .expect("Failed to read ZIP archive");

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("[Document] ZIP: File: {}", file.name());
    }

    let styles_document_text = load_archive_file_to_string(&mut archive, "word/styles.xml");
    let styles_document = xml::Document::parse(&styles_document_text)
            .expect("Failed to parse document");
    let style_manager = StyleManager::from_document(&styles_document).unwrap();
    
    let document_text = load_archive_file_to_string(&mut archive, "word/document.xml");
    let document = xml::Document::parse(&document_text)
            .expect("Failed to parse document");
    word_processing::process_document(&document, &style_manager)
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

struct Application {
    archive_path: String,

    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,

    window: RenderWindow,

    is_draw_invalidated: Arc<AtomicBool>,
    scroller: Scroller,
}

impl Application {
    fn new(archive_path: String) -> Self {
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
    
        watcher.watch(std::path::Path::new(&archive_path), RecursiveMode::NonRecursive).unwrap();

        let context_settings = ContextSettings::default();
        let mut window = RenderWindow::new(VideoMode::new(1280, 720, 32), 
                "Uffice", Style::DEFAULT, &context_settings);

        window.set_framerate_limit(30);

        Application {
            archive_path: archive_path.clone(),
            watcher,
            window,
            is_draw_invalidated,
            scroller: Scroller::new()
        }
    }

    fn run(&mut self) {
        let mut texture = sfml::graphics::RenderTexture::new(1, 1).unwrap();

        let mut shape = sfml::graphics::RectangleShape::new();
        let mut left_button_pressed = false;
        let mut mouse_position = Vector2f::new(0.0, 0.0);

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

                                if self.scroller.bar_rect.contains(mouse_position) {
                                    left_button_pressed = true;
                                }
                            }
                        }
                        Event::MouseButtonReleased { button, x: _, y: _ } => {
                            if button == sfml::window::mouse::Button::Left {
                                left_button_pressed = false;
                            }
                        }
                        Event::MouseMoved { x, y } => {
                            if left_button_pressed {
                                self.scroller.apply_mouse_offset(y as f32 - mouse_position.y);
                            }
                            
                            mouse_position = Vector2f::new(x as f32, y as f32);
                        }
                        _ => (),
                    }
                }
            }
            
            if self.is_draw_invalidated.swap(false, Ordering::Relaxed) {
                texture = draw_document(&self.archive_path);
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

                let scale = full_size * factor / page_size;
                let centered_x = (full_size - page_size * scale) / 2.0;
                sprite.set_scale((scale, scale));
            
                sprite.set_position((
                    centered_x,
                    20.0f32 - self.scroller.offset(sprite.texture_rect().height as f32)
                ));

                self.scroller.document_height = sprite.global_bounds().height;

                self.window.draw(&sprite);
            }

            // Scrollbar
            self.scroller.draw(&mut shape, &mut self.window);
    
            self.window.display();
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Uffice")]
struct Options {

}

fn main() {
    dotenv::dotenv().expect("Failed to load .env");

    println!(">> Uffice <<");

    let mut app = Application::new(
            std::env::var("UFFICE_TEST_FILE").expect("No file given")
    );
    app.run();
}
