// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;

#[derive(Debug)]
pub enum Error {
    XmlError(xml::Error),
    ParseIntError(std::num::ParseIntError),
    StyleNotFound,
}

impl From<xml::Error> for Error {
    fn from(error: xml::Error) -> Self {
        Self::XmlError(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}
