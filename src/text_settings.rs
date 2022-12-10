// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::graphics::{Color, TextStyle};

#[derive(Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct TextSettings {
    pub bold: bool,
    pub font: String,
    pub color: Color,

    pub spacing_below_paragraph: f32,
    pub non_complex_text_size: u32,
}

impl TextSettings {
    pub fn new(font: String) -> Self {
        Self{ 
            bold: false,
            font,
            color: Color::BLACK,
            spacing_below_paragraph: 0.0f32,
            non_complex_text_size: 22,
        }
    }

    pub fn resolve_font_file(self: &Self) -> String {
        println!("Font is \"{}\"", self.font);
        if self.font == "Times New Roman" {
            return String::from("C:/Windows/Fonts/times.ttf");
        }

        if self.bold {
            return String::from("C:/Windows/Fonts/calibrib.ttf");
        }

        String::from("C:/Windows/Fonts/calibri.ttf")
    }

    pub fn create_style(self: &Self) -> TextStyle {
        if self.bold {
            return TextStyle::BOLD;
        }

        TextStyle::REGULAR
    }
}