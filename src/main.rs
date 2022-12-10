// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

mod color_parser;
mod error;
mod font;
mod style;
mod text_settings;
mod word_processing;

use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use dotenv;

use roxmltree as xml;

use sfml::graphics::*;
use sfml::system::Vector2f;
use sfml::window::*;

//use font_kit;
use notify::{Watcher, RecursiveMode};
use style::StyleManager;
use text_settings::TextSettings;

pub const WORD_PROCESSING_XML_NAMESPACE: &'static str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn apply_run_properties_for_paragraph_mark(element: &xml::Node, text_settings: &mut TextSettings) {
    assert_eq!(element.tag_name().name(), "rPr");

    for run_property in element.children() {
        println!("│  │  │  ├─ {}", run_property.tag_name().name());
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

fn load_archive_file_to_string<'a>(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Rc<String> {
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

struct Application {
    archive_path: String,

    #[allow(dead_code)]
    watcher: notify::ReadDirectoryChangesWatcher,

    window: RenderWindow,

    is_draw_invalidated: Arc<AtomicBool>,
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
            is_draw_invalidated: is_draw_invalidated.clone(),
        }
    }

    fn run(self: &mut Self) {
        let mut texture = sfml::graphics::RenderTexture::new(1, 1).unwrap();

        while self.window.is_open() {
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

                let full_size = self.window.size().x as f32;
                let page_size = sprite.texture_rect().width as f32;
                let factor = 1.0 / 5.0 * 4.0;

                let scale = full_size * factor / page_size;
                let centered_x = (full_size - page_size * scale) / 2.0;
                sprite.set_scale((scale, scale));
            
                sprite.set_position((
                    centered_x,
                    20.0f32
                ));

                self.window.draw(&sprite);
            }
    
            self.window.display();
        }
    }
}

fn main() {
    dotenv::dotenv().expect("Failed to load .env");

    println!(">> Uffice <<");

    // let source = font_kit::source::SystemSource::new();
    // for font in source.all_fonts().expect("Failed to iterate fonts") {
    //     match font {
    //         font_kit::handle::Handle::Path { path, ..} => {
    //             println!("Font: {}", path.display());
    //         }
    //         _ => ()
    //     }
    // }

    
    let mut app = Application::new(
            std::env::var("UFFICE_TEST_FILE").expect("No file given")
    );
    app.run();
}
