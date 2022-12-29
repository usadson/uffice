// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{rc::Rc, cell::RefCell};

use font_kit::family_name::FamilyName;
use roxmltree as xml;
use sfml::{graphics::{Color, TextStyle, Font, Text}, system::Vector2f};

use crate::{word_processing::HALF_POINT, color_parser, WORD_PROCESSING_XML_NAMESPACE, style::StyleManager};

#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Rect {
    pub fn new(position: Vector2f, size: Vector2f) -> Self {
        Self {
            left: position.x as u32,
            right: (position.x + size.x) as u32,

            top: position.y as u32,
            bottom: (position.y + size.y) as u32,
        }
    }

    pub fn empty() -> Self {
        Rect { left: 0, right: 0, top: 0, bottom: 0 }
    }

    pub fn is_inside_inclusive(&self, position: Position) -> bool {
        position.x >= self.left && position.x <= self.right
            && position.y >= self.top && position.y <= self.bottom
    }
}

impl From<sfml::graphics::Rect<f32>> for Rect {
    fn from(some: sfml::graphics::Rect<f32>) -> Self {
        Self{
            left: some.left as u32,
            right: (some.left + some.width) as u32,

            top: some.top as u32,
            bottom: (some.top + some.height) as u32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PageSettings {
    pub size: Size,
    pub margins: Rect,
    pub offset_header: u32,
    pub offset_footer: u32,
}

impl PageSettings {
    pub fn new(size: Size, margins: Rect, offset_header: u32, offset_footer: u32) -> Self {
        Self { size, margins, offset_header, offset_footer }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextJustification {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone)]
pub struct Numbering {
    pub definition: Option<Rc<RefCell<crate::wp::numbering::NumberingDefinitionInstance>>>,
    pub level: Option<i32>,
}
impl Numbering {
    pub fn create_node(&self, paragraph: Rc<RefCell<crate::wp::Node>>) -> Rc<RefCell<crate::wp::Node>> {
        assert!(paragraph.try_borrow_mut().is_ok());
        let numbering_definition_instance = &self.definition
                .as_ref()
                .unwrap().as_ref().borrow_mut();
        let abstract_definition = numbering_definition_instance
                .abstract_numbering_definition
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow_mut();

        let level = abstract_definition.levels.get(&self.level.unwrap()).unwrap().as_ref().borrow();

        let mut node = crate::wp::Node::new(crate::wp::NodeData::TextPart(crate::wp::TextPart{
            text: self.formatted(),
        }));

        node.text_settings = self.combine_text_settings(&paragraph.as_ref().borrow(), &level);

        crate::wp::append_child(paragraph, node)
    }

    fn combine_text_settings(&self, paragraph: &crate::wp::Node, level: &crate::wp::numbering::NumberingLevelDefinition) -> TextSettings {
        let mut settings = paragraph.text_settings.clone();
        settings.inherit_from(&level.text_settings);
        settings
    }

    fn formatted(&self) -> String {
        String::new()
    }
}

#[derive(Clone, Debug)]
pub struct TextSettings {
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub font: Option<String>,
    pub color: Option<Color>,

    pub spacing_below_paragraph: Option<f32>,
    pub non_complex_text_size: Option<u32>,
    pub justify: Option<TextJustification>,

    pub highlight_color: Option<Color>,
    pub numbering: Option<Numbering>,

    /// Specifies the indentation which shall be removed from the first line of
    /// the parent paragraph, by moving the indentation on the first line back
    /// towards the beginning of the direction of text flow.
    pub indentation_hanging: Option<u32>,

    ///
    pub indentation_left: Option<u32>,
}

fn inherit_or_original<T: Clone>(inherit: &Option<T>, original: &mut Option<T>) {
    match inherit {
        Some(value) => *original = Some((*value).clone()),
        None => ()
    }
}

impl TextSettings {
    pub fn new() -> Self {
        Self{
            bold: None,
            underline: None,
            font: None,
            color: None,
            spacing_below_paragraph: None,
            non_complex_text_size: None,
            justify: None,
            highlight_color: None,
            numbering: None,
            indentation_hanging: None,
            indentation_left: None,
        }
    }

    pub fn inherit_from(&mut self, other: &TextSettings) {
        inherit_or_original(&other.bold, &mut self.bold);
        inherit_or_original(&other.underline, &mut self.underline);
        inherit_or_original(&other.font, &mut self.font);
        inherit_or_original(&other.color, &mut self.color);
        inherit_or_original(&other.spacing_below_paragraph, &mut self.spacing_below_paragraph);
        inherit_or_original(&other.non_complex_text_size, &mut self.non_complex_text_size);
        inherit_or_original(&other.justify, &mut self.justify);
        inherit_or_original(&other.highlight_color, &mut self.highlight_color);
        inherit_or_original(&other.numbering, &mut self.numbering);

        inherit_or_original(&other.indentation_hanging, &mut self.indentation_hanging);
        inherit_or_original(&other.indentation_left, &mut self.indentation_left);
    }

    pub fn load_font(&self, source: &font_kit::sources::multi::MultiSource) -> sfml::SfBox<Font> {
        let font: &str = match &self.font {
            Some(font) => font,
            None => "Calibri"
        };

        let mut properties = font_kit::properties::Properties::new();

        if self.bold.unwrap_or(false) {
            properties.weight = font_kit::properties::Weight::BOLD;
        }

        let family_names = [FamilyName::Title(String::from(font))];
        let handle = source.select_best_match(&family_names, &properties)
                .expect("Failed to find font");

        match handle {
            font_kit::handle::Handle::Memory { bytes, font_index: _ } => {
                unsafe {
                    Font::from_memory(&bytes).unwrap()
                }
            }
            font_kit::handle::Handle::Path { path, font_index: _ } => {
                Font::from_file(path.to_str().unwrap()).unwrap()
            }
        }
    }

    pub fn create_text<'a>(&self, font: &'a Font) -> Text<'a> {
        let character_size = match self.non_complex_text_size {
            Some(size) => size as f32 * HALF_POINT,
            None => panic!("No default text size defined!")
        } as u32;

        let mut text = Text::new("L", font, character_size);
        text.set_style(self.create_style());
        text.set_fill_color(self.color.unwrap_or(Color::BLACK));

        text
    }

    pub fn create_style(&self) -> TextStyle {
        let mut style = TextStyle::REGULAR;

        if self.bold.unwrap_or(false) {
            style |= TextStyle::BOLD;
        }

        if self.underline.unwrap_or(false) {
            style |= TextStyle::UNDERLINED;
        }

        style
    }

    pub fn apply_run_properties_element(&mut self, style_manager: &StyleManager, element: &xml::Node) {
        assert_eq!(element.tag_name().name(), "rPr");

        for run_property in element.children() {
            println!("│  │  │  ├─ {}", run_property.tag_name().name());
            for attr in run_property.attributes() {
                println!("│  │  │  │  ├─ Attribute \"{}\" => \"{}\"", attr.name(), attr.value());
            }

            match run_property.tag_name().name() {
                "b" => {
                    self.bold = match self.bold {
                        None => Some(true),
                        Some(bold) => Some(!bold)
                    };
                }
                "color" => {
                    for attr in run_property.attributes() {
                        println!("│  │  │  │  ├─ Color Attribute: {} => {}", attr.name(), attr.value());
                        if attr.name() == "val" && attr.value() != "auto" {
                            self.color = Some(color_parser::parse_color(attr.value()).unwrap());
                        }
                    }
                }

                // 17.3.2.15 highlight (Text Highlighting)
                "highlight" => {
                    let val = run_property.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                            .expect("No w:val on a <w:highlight> element!");
                    self.highlight_color = Some(color_parser::parse_highlight_color(val));
                }

                "rFonts" => {
                    for attr in run_property.attributes() {
                        println!("│  │  │  │  ├─ Font Attribute: {} => {}", attr.name(), attr.value());
                        if attr.name() == "ascii" {
                            self.font = Some(String::from(attr.value()));
                        }
                    }
                }

                "rStyle" => {
                    let val = run_property.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                            .expect("No w:val on a <w:highlight> element!");
                    style_manager.apply_character_style(val, self);
                }

                "sz" => {
                    for attr in run_property.attributes() {
                        println!("│  │  │  │  ├─ Size Attribute: {} => {}", attr.name(), attr.value());
                        if attr.name() == "val" {
                            let new_value = str::parse::<u32>(attr.value()).expect("Failed to parse attribute");
                            println!("│  │  │  │  ├─ Value Attribute: old={:?} new={}", self.non_complex_text_size, new_value);
                            self.non_complex_text_size = Some(new_value);
                        }
                    }
                }

                "u" => {
                    // TODO add more types (dash, dotted, etc.)
                    self.underline = match self.underline {
                        None => Some(true),
                        Some(underline) => Some(!underline)
                    };
                }
                _ => ()
            }
        }
    }

}
