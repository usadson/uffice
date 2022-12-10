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
