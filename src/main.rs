// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

mod application;
mod color_parser;
mod error;
mod style;
mod text_settings;
mod word_processing;

use roxmltree as xml;

use sfml::graphics::*;

use structopt::StructOpt;
use style::StyleManager;
use text_settings::TextSettings;

use crate::application::Application;

pub const WORD_PROCESSING_XML_NAMESPACE: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

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
