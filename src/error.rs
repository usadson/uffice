// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;

#[derive(Debug)]
pub enum Error {
    RoXmlTree(xml::Error),
    StdNumParseInt(std::num::ParseIntError),
    StyleNotFound,
}

impl From<xml::Error> for Error {
    fn from(error: xml::Error) -> Self {
        Self::RoXmlTree(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::StdNumParseInt(error)
    }
}
