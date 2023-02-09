// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::num::ParseIntError;

use uffice_lib::TwelfteenthPoint;

use crate::{
    style::{
        BorderProperties,
        BorderPropertiesParseError,
    },
    serialize::FromXmlStandalone, WORD_PROCESSING_XML_NAMESPACE,
};

#[derive(Clone, Debug,  Default)]
pub struct GridColumnDefinition {
    pub width: TwelfteenthPoint<u32>,
}

impl FromXmlStandalone for GridColumnDefinition {
    type ParseError = ParseIntError;

    fn from_xml(node: &roxmltree::Node) -> Result<Self, Self::ParseError>
            where Self: Sized {
        Ok(Self {
            width: match node.attribute((WORD_PROCESSING_XML_NAMESPACE, "w")) {
                Some(width) => TwelfteenthPoint(width.parse()?),
                None => TwelfteenthPoint(0)
            }
        })
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TableBorderProperties {
    pub top: BorderProperties,
    pub left: BorderProperties,
    pub bottom: BorderProperties,
    pub right: BorderProperties,
    pub inside_horizontal: BorderProperties,
    pub inside_vertical: BorderProperties,
}

#[derive(Clone, Debug, Default)]
pub struct TableGrid(pub Vec<GridColumnDefinition>);

impl FromXmlStandalone for TableGrid {
    type ParseError = ParseIntError;

    fn from_xml(node: &roxmltree::Node) -> Result<Self, Self::ParseError>
            where Self: Sized {
        let mut grid = Vec::new();

        for child in node.children() {
            if child.tag_name().name() == "gridCol" {
                grid.push(GridColumnDefinition::from_xml(&child)?);
            } else {
                println!("[WARNING] Invalid table grid child: \"{}\"", child.tag_name().name());
            }
        }

        Ok(Self(grid))
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TableProperties {
    pub borders: TableBorderProperties,
}

#[derive(Debug)]
pub enum TablePropertiesParseError {
    UnknownTableProperty(String),
    BorderPropertiesParseError(BorderPropertiesParseError),
}

impl From<BorderPropertiesParseError> for TablePropertiesParseError {
    fn from(error: BorderPropertiesParseError) -> Self {
        TablePropertiesParseError::BorderPropertiesParseError(error)
    }
}

impl FromXmlStandalone for TableProperties {
    type ParseError = TablePropertiesParseError;
    fn from_xml(node: &roxmltree::Node) -> Result<Self, TablePropertiesParseError>
            where Self: Sized {
        let mut properties = TableProperties::default();

        for child in node.children() {
            match child.tag_name().name() {
                "tblBorders" => {
                    for border in child.children() {
                        match border.tag_name().name() {
                            "top" => properties.borders.top = BorderProperties::from_xml(&border)?,
                            "left" => properties.borders.left = BorderProperties::from_xml(&border)?,
                            "bottom" => properties.borders.bottom = BorderProperties::from_xml(&border)?,
                            "right" => properties.borders.right = BorderProperties::from_xml(&border)?,
                            "insideH" => properties.borders.inside_horizontal = BorderProperties::from_xml(&border)?,
                            "insideV" => properties.borders.inside_vertical = BorderProperties::from_xml(&border)?,
                            _ => ()//return Err(TablePropertiesParseError::UnknownTableProperty(border.tag_name().name().to_string()))
                        }
                    }
                }
                _ => ()
                //_ => return Err(TablePropertiesParseError::UnknownTableProperty(child.tag_name().name().to_string()))
            }
        }

        Ok(properties)
    }
}
