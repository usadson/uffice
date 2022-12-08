// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::graphics::{Color, TextStyle};

#[derive(Clone)]
pub struct TextSettings {
    pub bold: bool,
    pub font: String,
    pub color: Color,

    pub spacing_below_paragraph: f32,
}

impl TextSettings {
    pub fn new(font: String) -> Self {
        Self{ 
            bold: false,
            font,
            color: Color::BLACK,
            spacing_below_paragraph: 0.0f32
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