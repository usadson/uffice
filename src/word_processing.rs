/**
 * Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
 * All Rights Reserved.
 */

use roxmltree as xml;
use sfml::{graphics::{Font, Text, Color}, system::Vector2f};

use crate::*;

//pub const WORD_PROCESSING_XML_NAMESPACE: &'static str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
pub const QUALITY: u32 = 8;
pub const QUALITY_PIXELS: u32 = QUALITY * 4;
const FACTOR: f32 = QUALITY_PIXELS as f32;

pub fn process_document(document: &xml::Document, render_texture: &mut sfml::graphics::RenderTexture) {
    let mut position = Vector2f::new(20.0 * FACTOR, 20.0 * FACTOR);

    let text_settings = TextSettings::new(String::from("Calibri"));

    for child in document.root_element().children() {
        println!("{}", child.tag_name().name());

        if child.tag_name().name() == "body" {
            position = process_body_element(&child, position, &text_settings, render_texture);
        }
    }
}

fn process_body_element(node: &xml::Node, 
                        position: Vector2f, 
                        text_settings: &crate::text_settings::TextSettings, 
                        render_texture: &mut sfml::graphics::RenderTexture) -> Vector2f {
    let mut position = position;

    for child in node.children() {
        println!("├─ {}", child.tag_name().name());
        if child.tag_name().name() == "p" {
            position = process_pragraph_element(&child, position, text_settings, render_texture);
        }
    }

    position
}

fn process_pragraph_element(node: &xml::Node, 
                            position: Vector2f, 
                            text_settings: &crate::text_settings::TextSettings, 
                            render_texture: &mut sfml::graphics::RenderTexture) -> Vector2f {
    let mut position = position;

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
                position = process_text_run_element(&child, position, &paragraph_text_settings, render_texture);
            }
            _ => ()
        }
    }

    position.x = 20.0 * FACTOR;

    let font = Font::from_file(&paragraph_text_settings.resolve_font_file())
                .expect("Failed to load font");

    position.y += font.line_spacing(30 * QUALITY) + paragraph_text_settings.spacing_below_paragraph;

    position
}

// pPr
fn process_paragraph_properties_element(node: &xml::Node, paragraph_text_settings: &mut TextSettings) {
    for property in node.children() {
        println!("│  │  ├─ {}", property.tag_name().name());

        if property.tag_name().name() == "spacing" {
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

        // Run Properties section 17.3.2.28
        if property.tag_name().name() == "rPr" {
            //apply_run_properties_for_paragraph_mark(&property, paragraph_text_settings); 
        }
    }
}

/// Process the w:t element.
fn process_text_element(node: &xml::Node, 
                        position: Vector2f, 
                        run_text_settings: &crate::text_settings::TextSettings, 
                        render_texture: &mut sfml::graphics::RenderTexture) -> Vector2f {
    let mut position = position;

    for child in node.children() {
        if child.node_type() == xml::NodeType::Text {
            let font = Font::from_file(&run_text_settings.resolve_font_file())
                .expect("Failed to load font");

            let mut text = Text::new(child.text().unwrap(), &font, 30 * QUALITY);
            text.set_fill_color(Color::BLACK);
            text.set_position(position);
            text.set_style(run_text_settings.create_style());
            text.set_fill_color(run_text_settings.color);

            position.x += text.local_bounds().width;

            println!("│  │  │  ├─ Text: {}", child.text().unwrap());
            render_texture.draw(&text);
        }
    }

    position
}

fn process_text_run_element(node: &xml::Node, 
                            position: Vector2f, 
                            paragraph_text_settings: &TextSettings, 
                            render_texture: &mut sfml::graphics::RenderTexture) -> Vector2f {
    let mut run_text_settings = paragraph_text_settings.clone();

    let mut position = position;

    for text_run_property in node.children() {
        println!("│  │  ├─ {}", text_run_property.tag_name().name());

        if text_run_property.tag_name().name() == "rPr" {
            apply_run_properties_for_paragraph_mark(&text_run_property, &mut run_text_settings);
        }

        if text_run_property.tag_name().name() == "t" {
            position = process_text_element(&text_run_property, position, &run_text_settings, render_texture);
        }
    }

    position
}
