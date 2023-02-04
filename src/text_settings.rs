// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{rc::Rc, cell::RefCell};

use roxmltree as xml;
use uffice_lib::TwelfteenthPoint;

use crate::{
    color_parser,
    WORD_PROCESSING_XML_NAMESPACE,
    style::StyleManager,
    wp::{
        layout::LineLayout,
        Node,
    },
    gui::Color,
    gui::{
        painter::{
            FontWeight,
            TextCalculator, FontStyle,
        },
        Rect,
        Size,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct PageSettings {
    pub size: Size<TwelfteenthPoint<u32>>,
    pub margins: Rect<TwelfteenthPoint<u32>>,
    pub offset_header: TwelfteenthPoint<u32>,
    pub offset_footer: TwelfteenthPoint<u32>,
}

impl PageSettings {
    pub fn new(size: Size<TwelfteenthPoint<u32>>, margins: Rect<TwelfteenthPoint<u32>>,
               offset_header: TwelfteenthPoint<u32>, offset_footer: TwelfteenthPoint<u32>) -> Self {
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
    pub fn create_node(&self, paragraph: &mut Node, line_layout: &mut LineLayout,
                       text_calculator: &mut dyn TextCalculator) -> (usize, usize) {
        let numbering_definition_instance = &self.definition
                .as_ref()
                .unwrap().as_ref().borrow_mut();
        let abstract_definition = numbering_definition_instance
                .abstract_numbering_definition
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow_mut();

        let level_idx = self.level.unwrap();
        let mut level = abstract_definition.levels.get(&level_idx).unwrap().borrow_mut();
        let numbering_value = level.next_value();

        let mut displayed_text = format!("{}.", level.format(numbering_value));
        for i in level_idx..0 {
            let level = abstract_definition.levels.get(&i).unwrap().as_ref().borrow();
            displayed_text = format!("{}.{}", displayed_text, level.format(level.current_value()));
        }

        // See the documentation of NodeData::NumberingParent for why we need
        // this parent and not just inherit from the parent Paragraph.
        let numbering_parent = crate::wp::create_child(paragraph, crate::wp::NodeData::NumberingParent);
        let text_settings = self.combine_text_settings(paragraph, &level);

        let num_parent = paragraph.nth_child_mut(numbering_parent);
        num_parent.text_settings = text_settings;

        crate::word_processing::append_text_element(&displayed_text, num_parent, line_layout, text_calculator);
        (numbering_parent, 0)
    }

    fn combine_text_settings(&self, paragraph: &crate::wp::Node, level: &crate::wp::numbering::NumberingLevelDefinition) -> TextSettings {
        let mut settings = paragraph.text_settings.clone();
        settings.inherit_from(&level.text_settings);
        settings
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
    pub indentation_hanging: Option<TwelfteenthPoint<i32>>,

    ///
    pub indentation_left: Option<TwelfteenthPoint<i32>>,
}

fn inherit_or_original<T: Clone + std::fmt::Debug>(inherit: &Option<T>, original: &mut Option<T>) {
    if let Some(value) = inherit {
        *original = Some((*value).clone());
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

    pub fn create_style(&self) -> FontStyle {
        let mut style = FontStyle::NORMAL;

        if self.bold.unwrap_or(false) {
            style |= FontStyle::BOLD;
        }

        if self.underline.unwrap_or(false) {
            style |= FontStyle::UNDERLINE;
        }

        style
    }

    pub fn apply_run_properties_element(&mut self, style_manager: &StyleManager, element: &xml::Node) {
        assert_eq!(element.tag_name().name(), "rPr");

        for run_property in element.children() {
            match run_property.tag_name().name() {
                "b" => {
                    self.bold = match self.bold {
                        None => Some(true),
                        Some(bold) => Some(!bold)
                    };
                }
                "color" => {
                    for attr in run_property.attributes() {
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
                        //println!("│  │  │  │  ├─ Font Attribute: {} => {}", attr.name(), attr.value());
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
                        //println!("│  │  │  │  ├─ Size Attribute: {} => {}", attr.name(), attr.value());
                        if attr.name() == "val" {
                            let new_value = str::parse::<u32>(attr.value()).expect("Failed to parse attribute");
                            //println!("│  │  │  │  ├─ Value Attribute: old={:?} new={}", self.non_complex_text_size, new_value);
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

    pub(crate) fn indent_one(&self, x: f32, _is_first_line: bool) -> f32 {
        //println!("indent_one");
        //println!("  In X: {}", x);

        if let Some(indentation) = self.indentation_left {
            //println!("  IndentationLeft: {}", indentation);

            let indentation = indentation.get_pts();
            //println!("  Step: {}", indentation);

            let x = ((x / indentation) as u32 + 1) as f32 * indentation;
            //println!("  X: {}", x);

            // return if let Some(hanging) = self.indentation_hanging {
            //     x - hanging as f32 * TWELFTEENTH_POINT
            // } else {
            //     x
            // }
            return x;
        }

        x
    }

    pub fn parse_element_ind(&mut self, node: &xml::Node) {
        // The w:left is a MSOFFICE quirk I believe, it isn't part
        // of the ECMA/ISO standard.
        if let Some(value) = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "left")) {
            self.indentation_left = Some(TwelfteenthPoint(value.parse().unwrap()));
        }

        if let Some(value) = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "hanging")) {
            self.indentation_hanging = Some(TwelfteenthPoint(value.parse().unwrap()));
        }
    }

    pub fn font_weight(&self) -> FontWeight {
        if self.bold == Some(true) {
            FontWeight::Bold
        } else {
            FontWeight::Regular
        }
    }

    pub fn brush(&self) -> crate::gui::Brush {
        let color = self.color.unwrap_or(Color::BLACK);
        crate::gui::Brush::SolidColor(color)
    }

}
