// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::graphics::Color;

#[derive(Debug)]
pub enum ColorParseError {
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

    if (DIGIT_0..=DIGIT_9).contains(&c) {
        return Ok(c - DIGIT_0);
    }

    if (ALPHA_UPPER_A..=ALPHA_UPPER_F).contains(&c) {
        return Ok(c - ALPHA_UPPER_A + 0xA);
    }

    if (ALPHA_LOWER_A..=ALPHA_LOWER_F).contains(&c) {
        return Ok(c - ALPHA_LOWER_A + 0xA);
    }

    Err(ColorParseError::ElementNotHexCharacter)
}

fn parse_color_element(a: u8, b: u8) -> Result<u8, ColorParseError> {
    Ok(parse_color_element_hex_character(a)? << 4 | parse_color_element_hex_character(b)?)
}

pub fn parse_color(value: &str) -> Result<Color, ColorParseError> {
    if value.len() != 6 {
        return Err(ColorParseError::LengthNotSixBytes);
    }

    Ok(Color::rgb(
        parse_color_element(value.as_bytes()[0], value.as_bytes()[1])?,
        parse_color_element(value.as_bytes()[2], value.as_bytes()[3])?,
        parse_color_element(value.as_bytes()[4], value.as_bytes()[5])?
    ))
}

pub fn parse_highlight_color(value: &str) -> Color {
    match value {
        "black" => Color::rgb(0, 0, 0),
        "blue" => Color::rgb(0, 0, 0xFF),
        "cyan" => Color::rgb(0, 0xFF, 0xFF),
        "darkBlue" => Color::rgb(0, 0, 0x8B),
        "darkCyan" => Color::rgb(0, 0x8B, 0x8B),
        "darkGray" => Color::rgb(0xA9, 0xA9, 0xA9),
        "darkGreen" => Color::rgb(0, 0x64, 0),
        "darkMagenta" => Color::rgb(0x80, 0, 0x80),
        "darkRed" => Color::rgb(0x8B, 0, 0),
        "darkYellow" => Color::rgb(0x80, 0x80, 0),
        "green" => Color::rgb(0, 0xFF, 0),
        "lightGray" => Color::rgb(0xD3, 0xD3, 0xD3),
        "magenta" => Color::rgb(0xFF, 0, 0xFF),
        "none" => Color::rgba(0, 0, 0, 0),
        "red" => Color::rgb(0xFF, 0, 0),
        "white" => Color::rgb(0xFF, 0xFF, 0xFF),
        "yellow" => Color::rgb(0xFF, 0xFF, 0),
        _ => {
            panic!("Invalid ST_HighlightColor: \"{}\"", value);
        }
    }
}
