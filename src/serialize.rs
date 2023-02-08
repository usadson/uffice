// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;

/// Parse the type from xml without references to other structures.
pub trait FromXmlStandalone {
    type ParseError;

    /// Parse the type from xml without references to other structures.
    fn from_xml(node: &xml::Node) -> Result<Self, Self::ParseError>
        where Self: Sized;
}
