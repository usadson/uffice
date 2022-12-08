/**
 * Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
 * All Rights Reserved.
 */

mod font;

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use dotenv;

use roxmltree as xml;

use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;

//use font_kit;
use notify::{Watcher, RecursiveMode};

#[derive(Clone)]
struct TextSettings {
    pub bold: bool,
    pub font: String,
    pub color: Color,
}

impl TextSettings {
    fn new(font: String) -> Self {
        Self{ 
            bold: false,
            font,
            color: Color::BLACK,
        }
    }

    fn resolve_font_file(self: &Self) -> String {
        println!("Font is \"{}\"", self.font);
        if self.font == "Times New Roman" {
            return String::from("C:/Windows/Fonts/times.ttf");
        }

        if self.bold {
            return String::from("C:/Windows/Fonts/calibrib.ttf");
        }

        String::from("C:/Windows/Fonts/calibri.ttf")
    }

    fn create_style(self: &Self) -> TextStyle {
        if self.bold {
            return TextStyle::BOLD;
        }

        TextStyle::REGULAR
    }
}

fn apply_run_properties_for_paragraph_mark(element: &xml::Node, text_settings: &mut TextSettings) {
    for run_property in element.children() {
        println!("│  │  ├─ {}", run_property.tag_name().name());
        match run_property.tag_name().name() {
            "b" => {
                println!("Set to bold: was={} new={}", text_settings.bold, !text_settings.bold);
                text_settings.bold = !text_settings.bold;
            }
            "color" => {
                for attr in run_property.attributes() {
                    println!("│  │  │  ├─ Color Attribute: {} => {}", attr.name(), attr.value());
                    if attr.name() == "val" {
                        text_settings.color = parse_color(attr.value()).unwrap();
                    }
                }
            }
            "rFonts" => {
                for attr in run_property.attributes() {
                    println!("│  │  │  ├─ Font Attribute: {} => {}", attr.name(), attr.value());
                    if attr.name() == "ascii" {
                        text_settings.font = String::from(attr.value());
                    }
                }
            }
            _ => ()
        }
    }
}

#[derive(Debug)]
enum ColorParseError {
    LengthNotSixBytes,
    ElementNotHexCharacter,
}

fn parse_color_element_hex_character(c: u8) -> Result<u8, ColorParseError> {
    const DIGIT_0: u8 = 0x30;
    const DIGIT_9: u8 = 0x39;

    const ALPHA_UPPER_A: u8 = 0x41;
    const ALPHA_UPPER_F: u8 = 0x46;

    const ALPHA_LOWER_A: u8 = 0x61;
    const ALPHA_LOWER_F: u8 = 0x66;

    if c >= DIGIT_0 && c <= DIGIT_9 {
        return Ok(c - DIGIT_0);
    }

    if c >= ALPHA_UPPER_A && c <= ALPHA_UPPER_F {
        return Ok(c - ALPHA_UPPER_A + 0xA);
    }

    if c >= ALPHA_LOWER_A && c <= ALPHA_LOWER_F {
        return Ok(c - ALPHA_LOWER_A + 0xA);
    }

    Err(ColorParseError::ElementNotHexCharacter)
}

fn parse_color_element(a: u8, b: u8) -> Result<u8, ColorParseError> {
    Ok(parse_color_element_hex_character(a)? << 4 | parse_color_element_hex_character(b)?)
}

fn parse_color(value: &str) -> Result<Color, ColorParseError> {
    if value.len() != 6 {
        return Err(ColorParseError::LengthNotSixBytes);
    }

    Ok(Color::rgb(
        parse_color_element(value.as_bytes()[0], value.as_bytes()[1])?,
        parse_color_element(value.as_bytes()[2], value.as_bytes()[3])?,
        parse_color_element(value.as_bytes()[4], value.as_bytes()[5])?
    ))
}

const WORD_PROCESSING_XML_NAMESPACE: &'static str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const QUALITY: u32 = 8;
const QUALITY_PIXELS: u32 = QUALITY * 4;

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

    let zip_document = archive.by_name("word/document.xml").expect("Not a DOCX file");
    let document_text = Rc::new(std::io::read_to_string(zip_document)
            .expect("Failed to read"));
    
    let document = xml::Document::parse(&document_text)
        .expect("Failed to parse document");

    let mut render_texture = RenderTexture::new(210 * QUALITY_PIXELS, 297 * QUALITY_PIXELS)
        .expect("Failed to create RenderTexture");

    let factor = QUALITY_PIXELS as f32;

    render_texture.clear(Color::WHITE);

    let mut position = Vector2f::new(20.0 * factor, 20.0 * factor);

    let text_settings = TextSettings::new(String::from("Calibri"));

    for paragraph in document.descendants() {
        println!("{}", paragraph.tag_name().name());
        if paragraph.tag_name().name() != "p" {
            continue;
        }

        let mut paragraph_text_settings = text_settings.clone();

        for paragraph_child in paragraph.children() {
            println!("├─ {}", paragraph_child.tag_name().name());
            // Paragraph Properties section 17.3.1.26
            if paragraph_child.tag_name().name() == "pPr" {
                let paragraph_properties = paragraph_child;

                for paragraph_property in paragraph_properties.children() {
                    println!("│  ├─ {}", paragraph_property.tag_name().name());
                    // Run Properties section 17.3.2.28
                    if paragraph_property.tag_name().name() == "rPr" {
                       apply_run_properties_for_paragraph_mark(&paragraph_property, &mut paragraph_text_settings); 
                    }
                }
            }

            // Text Run
            if paragraph_child.tag_name().name() == "r" {
                let text_run = paragraph_child;

                let mut run_text_settings = paragraph_text_settings.clone();

                for text_run_property in text_run.children() {
                    println!("│  ├─ {}", text_run_property.tag_name().name());

                    if text_run_property.tag_name().name() == "rPr" {
                        apply_run_properties_for_paragraph_mark(&text_run_property, &mut run_text_settings);
                    }

                    if text_run_property.tag_name().name() == "t" {
                        for child in text_run_property.children() {
                            if child.node_type() == xml::NodeType::Text {
                                let font = Font::from_file(&run_text_settings.resolve_font_file())
                                    .expect("Failed to load font");
                                

                                let mut text = Text::new(child.text().unwrap(), &font, 30 * QUALITY);
                                text.set_fill_color(Color::BLACK);
                                text.set_position(position);
                                text.set_style(run_text_settings.create_style());
                                text.set_fill_color(run_text_settings.color);

                                position.x += text.local_bounds().width;

                                println!("Text: {}", child.text().unwrap());
                                render_texture.draw(&text);
                            }
                        } 
                    }       
                }
            }
        }


    }

    render_texture.display();
    render_texture.set_smooth(true);
    return render_texture;
}
struct Application {
    archive_path: String,
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
    
                let factor = 1.0 / (QUALITY as f32);
                let centered_x = ((self.window.size().x as f32) - (sprite.texture_rect().width as f32) * factor) / 2.0;
                sprite.set_scale((factor, factor));
            
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
