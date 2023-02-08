// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::rc::Rc;

use crate::serialize::FromXmlStandalone;

use roxmltree as xml;

#[derive(Debug)]
pub struct FontCollection {
    pub latin: TextFont,
    pub east_asian: TextFont,
    pub complex_script: TextFont,
}

impl Default for FontCollection {
    fn default() -> Self {
        let font = Rc::from(String::new());
        Self {
            latin: TextFont { typeface: Rc::clone(&font) },
            east_asian: TextFont { typeface: Rc::clone(&font) },
            complex_script: TextFont { typeface: font },
        }
    }
}

impl FromXmlStandalone for FontCollection {
    type ParseError = ParseError;

    fn from_xml(node: &xml::Node) -> Result<Self, Self::ParseError>
            where Self: Sized {
        let mut result = Self::default();

        for child in node.children() {
            match child.tag_name().name() {
                "latin" => {
                    result.latin = TextFont::from_xml(&child)?;
                }
                "ea" => {
                    result.east_asian = TextFont::from_xml(&child)?;
                }
                "cs" => {
                    result.complex_script = TextFont::from_xml(&child)?;
                }
                _ => {}
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct FontScheme {
    pub major_font: FontCollection,
    pub minor_font: FontCollection,
}

impl FromXmlStandalone for FontScheme {
    type ParseError = ParseError;

    fn from_xml(node: &xml::Node) -> Result<Self, Self::ParseError> {
        let mut major_font = FontCollection::default();
        let mut minor_font = FontCollection::default();

        for child in node.children() {
            match child.tag_name().name() {
                "majorFont" => {
                    major_font = FontCollection::from_xml(&child)?;
                }
                "minorFont" => {
                    minor_font = FontCollection::from_xml(&child)?;
                }
                _ => {}
            }
        }

        Ok(Self {
            major_font,
            minor_font,
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidXmlStructureRoot,
}

/// 20.1.4 Styles
#[derive(Debug, Default)]
pub struct StyleSettings {
    pub theme_elements: ThemeElements,
}

impl FromXmlStandalone for StyleSettings {
    type ParseError = ParseError;

    fn from_xml(node: &xml::Node) -> Result<Self, Self::ParseError> {
        let mut theme_elements = Default::default();

        if node.children().count() != 1 {
            return Err(ParseError::InvalidXmlStructureRoot);
        }

        let node = node.children().next().unwrap();
        if node.tag_name().name() != "theme" {
            return Err(ParseError::InvalidXmlStructureRoot);
        }

        for child in node.children() {
            if child.tag_name().name() == "themeElements" {
                theme_elements = ThemeElements::from_xml(&child)?;
            }
        }

        Ok(Self {
            theme_elements,
        })
    }
}

#[derive(Debug)]
pub struct TextFont {
    /// The typeface, or the empty string if no default is specified.
    pub typeface: Rc<str>,
}

impl FromXmlStandalone for TextFont {
    type ParseError = ParseError;

    fn from_xml(node: &xml::Node) -> Result<Self, Self::ParseError> {
        Ok(Self {
            typeface: Rc::from(node.attribute("typeface").unwrap_or_default()),
        })
    }
}

#[derive(Debug, Default)]
pub struct ThemeElements {
    pub font_scheme: FontScheme,
}

impl FromXmlStandalone for ThemeElements {
    type ParseError = ParseError;

    fn from_xml(node: &xml::Node) -> Result<Self, Self::ParseError> {
        let mut font_scheme = FontScheme::default();

        for child in node.children() {
            if child.tag_name().name() == "fontScheme" {
                font_scheme = FontScheme::from_xml(&child)?;
            }
        }

        Ok(Self {
            font_scheme,
        })
    }
}
