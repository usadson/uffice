// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use crate::gui::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorParseError {
    LengthNotSixBytes,
    ElementNotHexCharacter,
}

/// Parses a hex character:
/// 1. '0' to '9' inclusive => 0x0 to 0x9
/// 2. 'A' to 'F' inclusive => 0xA to 0xF
/// 2. 'a' to 'f' inclusive => 0xa to 0xf
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

    Ok(Color::from_rgb(
        parse_color_element(value.as_bytes()[0], value.as_bytes()[1])?,
        parse_color_element(value.as_bytes()[2], value.as_bytes()[3])?,
        parse_color_element(value.as_bytes()[4], value.as_bytes()[5])?
    ))
}

pub fn parse_highlight_color(value: &str) -> Color {
    match value {
        "black" => Color::from_rgb(0, 0, 0),
        "blue" => Color::from_rgb(0, 0, 0xFF),
        "cyan" => Color::from_rgb(0, 0xFF, 0xFF),
        "darkBlue" => Color::from_rgb(0, 0, 0x8B),
        "darkCyan" => Color::from_rgb(0, 0x8B, 0x8B),
        "darkGray" => Color::from_rgb(0xA9, 0xA9, 0xA9),
        "darkGreen" => Color::from_rgb(0, 0x64, 0),
        "darkMagenta" => Color::from_rgb(0x80, 0, 0x80),
        "darkRed" => Color::from_rgb(0x8B, 0, 0),
        "darkYellow" => Color::from_rgb(0x80, 0x80, 0),
        "green" => Color::from_rgb(0, 0xFF, 0),
        "lightGray" => Color::from_rgb(0xD3, 0xD3, 0xD3),
        "magenta" => Color::from_rgb(0xFF, 0, 0xFF),
        "none" => Color::from_rgba(0, 0, 0, 0),
        "red" => Color::from_rgb(0xFF, 0, 0),
        "white" => Color::from_rgb(0xFF, 0xFF, 0xFF),
        "yellow" => Color::from_rgb(0xFF, 0xFF, 0),
        _ => {
            panic!("Invalid ST_HighlightColor: \"{}\"", value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_element_hex_character() {
        for i in 0..=255 {
            // U+0030 '0' to U+0039 '9'
            if (0x30..0x3A).contains(&i) {
                assert_eq!(parse_color_element_hex_character(i), Ok(i - 0x30));
                continue;
            }

            // U+0041 'A' to U+0046 'F'
            if (0x41..0x47).contains(&i) {
                assert_eq!(parse_color_element_hex_character(i), Ok(i - 0x41 + 0xA));
                continue;
            }

            // U+0061 'a' to U+0066 'f'
            if (0x61..0x67).contains(&i) {
                assert_eq!(parse_color_element_hex_character(i), Ok(i - 0x61 + 0xA));
                continue;
            }

            assert_eq!(parse_color_element_hex_character(i), Err(ColorParseError::ElementNotHexCharacter));
        }
    }
}
