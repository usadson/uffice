/**
 * Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
 * All Rights Reserved.
 */

use roxmltree as xml;
use unicode_segmentation::{self, UnicodeSegmentation};

use sfml::{graphics::{Font, Text, Color}, system::Vector2f};

use crate::{*, text_settings::{PageSettings, Size}};

pub const WORD_PROCESSING_XML_NAMESPACE: &'static str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

const CORE_FACTOR: f32 = 2.0f32;
const TWELFTEENTH_POINT: f32 = 1f32 / 12.0 * CORE_FACTOR;
const HALF_POINT: f32 = 1f32 * CORE_FACTOR;
const LINE_SPACING: f32 = 6.0 * CORE_FACTOR;

struct Context<'a> {
    #[allow(dead_code)]
    document: &'a xml::Document<'a>,
    page_settings: PageSettings,

    #[allow(dead_code)]
    render_size: Vector2f,
    render_texture: &'a mut sfml::graphics::RenderTexture,
}

#[derive(Debug)]
enum Error {
    XmlError(xml::Error),
    ParseIntError(std::num::ParseIntError),
}

impl From<xml::Error> for Error {
    fn from(error: xml::Error) -> Self {
        Self::XmlError(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

fn load_page_settings(document: &xml::Document) -> Result<PageSettings, Error> {
    for root_child in document.root_element().first_child().unwrap().children() {
        println!("Direct Root Child {}", root_child.tag_name().name());
        if root_child.tag_name().name() != "sectPr" {
            continue;
        }

        let mut page_size = Size::new(10, 10);
        let mut margins = text_settings::Rect::empty();

        let mut offset_header = 0;
        let mut offset_footer = 0;

        for child in root_child.children() {
            match child.tag_name().name() {
                "pgSz" => {
                    page_size = Size::new(
                        str::parse(child.attribute((WORD_PROCESSING_XML_NAMESPACE, "w")).expect("No width parameter"))?,
                        str::parse(child.attribute((WORD_PROCESSING_XML_NAMESPACE, "h")).expect("No height parameter"))?
                    );
                }
                "pgMar" => {
                    for attribute in child.attributes() {
                        match attribute.name() {
                            "left" => margins.left = str::parse(attribute.value())?,
                            "right" => margins.right = str::parse(attribute.value())?,
                            "top" => margins.top = str::parse(attribute.value())?,
                            "bottom" => margins.bottom = str::parse(attribute.value())?,
                            "header" => offset_header = str::parse(attribute.value())?,
                            "footer" => offset_footer = str::parse(attribute.value())?,
                            _ => ()
                        }
                    }
                }
                _ => ()
            }
        }

        return Ok(PageSettings::new(page_size, margins, offset_header, offset_footer));
    }

    panic!("No direct child \"sectPr\" of root element found :(");
}

pub fn process_document(document: &xml::Document) -> sfml::graphics::RenderTexture {
    let text_settings = TextSettings::new(String::from("Calibri"));
    let page_settings = load_page_settings(document).unwrap();
    
    let mut position = Vector2f::new(
        page_settings.margins.left as f32 * TWELFTEENTH_POINT,
        page_settings.margins.top as f32 * TWELFTEENTH_POINT
    );

    println!("Rendering Document:\n\tSize: {} x {}\n\tRender Size: {} x {}",
        page_settings.size.width, 
        page_settings.size.height,
        page_settings.size.width as f32 / 12f32,
        page_settings.size.height as f32 / 12f32
    );

    let render_size = Vector2f::new(
        page_settings.size.width as f32 * TWELFTEENTH_POINT,
        page_settings.size.height as f32 * TWELFTEENTH_POINT
    );
    
    let mut render_texture = RenderTexture::new(render_size.x as u32, render_size.y as u32)
            .expect("Failed to create RenderTexture for document");

    render_texture.clear(Color::WHITE);

    let mut context = Context{
        document,
        page_settings,
        
        render_size,
        render_texture: &mut render_texture
    };

    for child in document.root_element().children() {
        println!("{}", child.tag_name().name());

        if child.tag_name().name() == "body" {
            position = process_body_element(&mut context, &child, position, &text_settings);
        }
    }

    render_texture.display();
    render_texture.set_smooth(true);

    render_texture
}

fn process_body_element(context: &mut Context,
                        node: &xml::Node, 
                        position: Vector2f, 
                        text_settings: &crate::text_settings::TextSettings) -> Vector2f {
    let mut position = position;

    for child in node.children() {
        println!("├─ {}", child.tag_name().name());
        if child.tag_name().name() == "p" {
            position = process_pragraph_element(context, &child, position, text_settings);
        }
    }

    position
}

fn process_pragraph_element(context: &mut Context,
                            node: &xml::Node, 
                            position: Vector2f, 
                            text_settings: &crate::text_settings::TextSettings) -> Vector2f {
    let mut position = position;

    position.x = context.page_settings.margins.left as f32 * TWELFTEENTH_POINT;

    let mut paragraph_text_settings = text_settings.clone();

    for child in node.children() {
        println!("│  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            // Paragraph Properties section 17.3.1.26
            "pPr" => {
                process_paragraph_properties_element(&child, &mut paragraph_text_settings);
            }
            // Text Run
            "r" => {
                position = process_text_run_element(context, &child, position, &paragraph_text_settings);
            }
            _ => ()
        }
    }

    let font = Font::from_file(&paragraph_text_settings.resolve_font_file())
                .expect("Failed to load font");

    position.y += font.line_spacing((paragraph_text_settings.non_complex_text_size as f32 * HALF_POINT) as u32) + paragraph_text_settings.spacing_below_paragraph;

    position
}

// pPr
fn process_paragraph_properties_element(node: &xml::Node, paragraph_text_settings: &mut TextSettings) {
    for property in node.children() {
        println!("│  │  ├─ {}", property.tag_name().name());

        for sub_property in property.children() {
            println!("│  │  │  ├─ {}", sub_property.tag_name().name());
            for attr in sub_property.attributes() {
                println!("│  │  │  │  ├─ A: {} = {}", attr.name(), attr.value());
            }
            for sub_property2 in sub_property.children() {
                println!("│  │  │  │  ├─ P: {}", sub_property2.tag_name().name());
            }
        }

        match property.tag_name().name() {
            // Run Properties section 17.3.2.28
            "rPr" => {
                //apply_run_properties_for_paragraph_mark(&property, paragraph_text_settings); 
            }
            "spacing" => {
                for attribute in property.attributes() {
                    println!("│  │  │  ├─ Spacing Attribute: {} = {}", attribute.name(), attribute.value());
                    match attribute.name() {
                        "after" => {
                            paragraph_text_settings.spacing_below_paragraph = str::parse(attribute.value())
                                    .expect("Failed to parse <w:spacing> 'after' attribute");
                        }
                        _ => ()
                    }
                }
            }
            _ => ()
        }
    }
}

/// Process the w:t element.
fn process_text_element(context: &mut Context,
                        node: &xml::Node, 
                        position: Vector2f, 
                        run_text_settings: &crate::text_settings::TextSettings) -> Vector2f {
    let mut position = position;

    for child in node.children() {
        if child.node_type() == xml::NodeType::Text {
            let font = Font::from_file(&run_text_settings.resolve_font_file())
                .expect("Failed to load font");

            let mut text = Text::new("", &font, (run_text_settings.non_complex_text_size as f32 * HALF_POINT) as u32);
            text.set_style(run_text_settings.create_style());
            text.set_fill_color(run_text_settings.color);
            
            let text_string = child.text().unwrap();
            println!("│  │  │  ├─ Text: \"{}\"", text_string);

            let page_horizontal_start = context.page_settings.margins.left as f32 * TWELFTEENTH_POINT;
            let page_horizontal_end = (context.page_settings.size.width - context.page_settings.margins.right) as f32 * TWELFTEENTH_POINT;

            let mut start_index = None;
            let mut previous_word_pair = None;

            let mut iter = UnicodeSegmentation::split_word_bound_indices(text_string).peekable();
            while let Some((index, word)) = iter.next() {
                let start;
                match start_index {
                    Some(start_index) => start = start_index,
                    None => {
                        start_index = Some(index);
                        start = index;
                    }
                }

                let max_width_fitting_on_page = page_horizontal_end - position.x;
                if max_width_fitting_on_page < 0.0 {
                    position.y += text.global_bounds().height + text.line_spacing() * LINE_SPACING;
                    position.x = page_horizontal_start;
                }

                let mut substring = &text_string[start..(index + word.chars().count())];
                text.set_string(substring);
                let mut width = text.global_bounds().width;

                if iter.peek().is_some() {
                    if width < max_width_fitting_on_page {
                        previous_word_pair = Some((index, word));
                        continue;
                    }

                    if let Some((previous_word_index, previous_word)) = previous_word_pair {
                        substring = &text_string[start..(previous_word_index + previous_word.chars().count())];
                        text.set_string(substring);
                        width = text.global_bounds().width;

                        start_index = Some(index);
                    } else {
                        start_index = None;
                    }
                }
                
                previous_word_pair = None;

                println!("│  │  │  │  ├─ Substring: \"{}\"", substring);
                println!("│  │  │  │  ├─ Calculation: x={} w={} m={}", position.x, width, max_width_fitting_on_page);
                text.set_position(position);

                context.render_texture.draw(&text);
                position.x += width;
            }

            assert!(previous_word_pair.is_none());
        }
    }

    position
}

fn process_text_run_element(context: &mut Context,
                            node: &xml::Node, 
                            position: Vector2f, 
                            paragraph_text_settings: &TextSettings) -> Vector2f {
    let mut run_text_settings = paragraph_text_settings.clone();

    let mut position = position;

    for text_run_property in node.children() {
        println!("│  │  ├─ {}", text_run_property.tag_name().name());

        if text_run_property.tag_name().name() == "rPr" {
            apply_run_properties_for_paragraph_mark(&text_run_property, &mut run_text_settings);
        }

        if text_run_property.tag_name().name() == "t" {
            position = process_text_element(context, &text_run_property, position, &run_text_settings);
        }
    }

    position
}
