// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;
use std::collections::HashMap;

use crate::{
    error::Error,
};

pub enum RelationshipType {
    CustomXml,
    FontTable,
    Hyperlink,
    Settings,
    Styles,
    Theme,
    WebSettings
}

impl RelationshipType {
    fn convert(name: &str) -> Option<Self> {
        match name {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" => Some(Self::CustomXml),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" => Some(Self::FontTable),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" => Some(Self::Hyperlink),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings" => Some(Self::WebSettings),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" => Some(Self::Styles),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" => Some(Self::Settings),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" => Some(Self::Theme),
            _ => None
        }
    }
}

pub struct Relationship {
    pub relation_type: RelationshipType,
    pub target: String,
}

pub struct Relationships {
    relationships: HashMap<String, Relationship>
}

impl Relationships {
    pub fn empty() -> Self {
        Self {
            relationships: HashMap::new()
        }
    }

    pub fn load_xml(document: &xml::Document) -> Result<Self, Error> {
        assert_eq!(document.root_element().tag_name().name(), "Relationships");

        let mut relationships = HashMap::new();

        for relationship_xml in document.root_element().children() {
            if relationship_xml.tag_name().name() != "Relationship" {
                continue;
            }

            relationships.insert(String::from(relationship_xml.attribute("Id").unwrap()), Relationship{
                relation_type: RelationshipType::convert(relationship_xml.attribute("Type").unwrap()).unwrap(),
                target: String::from(relationship_xml.attribute("Target").unwrap()),
            });
        }

        Ok(Self { 
            relationships
        })
    }

    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    pub fn find(&self, name: &str) -> Option<&Relationship> {
        self.relationships.get(name)
    }
}
