// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::graphics::{Color, TextStyle, Font, Text};

use crate::word_processing::HALF_POINT;

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Rect {
    pub fn empty() -> Self {
        Rect { left: 0, right: 0, top: 0, bottom: 0 }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone)]
pub struct TextSettings {
    pub bold: Option<bool>,
    pub font: Option<String>,
    pub color: Option<Color>,

    pub spacing_below_paragraph: Option<f32>,
    pub non_complex_text_size: Option<u32>,
    pub justify: Option<TextJustification>,
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
            font: None,
            color: None,
            spacing_below_paragraph: None,
            non_complex_text_size: None,
            justify: None,
        }
    }

    pub fn inherit_from(&mut self, other: &TextSettings) {
        inherit_or_original(&other.bold, &mut self.bold);
        inherit_or_original(&other.font, &mut self.font);
        inherit_or_original(&other.color, &mut self.color);
        inherit_or_original(&other.spacing_below_paragraph, &mut self.spacing_below_paragraph);
        inherit_or_original(&other.non_complex_text_size, &mut self.non_complex_text_size);
    }

    pub fn resolve_font_file(&self) -> String {
        let font: &str = match &self.font {
            Some(font) => font,
            None => "Calibri"
        };

        match font {
            "Calibri" => {
                if self.bold.unwrap_or(false) {
                    String::from("C:/Windows/Fonts/calibrib.ttf")
                } else {
                    String::from("C:/Windows/Fonts/calibri.ttf")
                }
            }
            "Times New Roman" => String::from("C:/Windows/Fonts/times.ttf"),
            _ => String::from("C:/Windows/Fonts/calibri.ttf")
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
        match self.bold {
            None => TextStyle::REGULAR,
            Some(bold) => match bold {
                true => TextStyle::BOLD,
                false => TextStyle::REGULAR
            }
        }
    }
}