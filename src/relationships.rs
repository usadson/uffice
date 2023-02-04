// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;
use zip::ZipArchive;
use std::{collections::HashMap, rc::Rc, cell::RefCell, fs::File};

use crate::{
    error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RelationshipType {
    Unknown,

    Comments,
    CommentsExtended,
    CommentsExtensible,
    CommentsIds,
    CustomXml,
    Endnotes,
    FontTable,
    Footer,
    Footnotes,
    GlossaryDocument,
    Header,
    Hyperlink,
    Image,
    Numbering,
    People,
    Settings,
    Styles,
    Theme,
    WebSettings
}

impl RelationshipType {
    fn convert(name: &str) -> Option<Self> {
        match name {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" => Some(Self::Comments),
            "http://schemas.microsoft.com/office/2011/relationships/commentsExtended" => Some(Self::CommentsExtended),
            "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible" => Some(Self::CommentsExtensible),
            "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds" => Some(Self::CommentsIds),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" => Some(Self::CustomXml),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes" => Some(Self::Endnotes),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" => Some(Self::FontTable),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" => Some(Self::Footer),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" => Some(Self::Footnotes),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/glossaryDocument" => Some(Self::GlossaryDocument),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" => Some(Self::Header),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" => Some(Self::Hyperlink),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" => Some(Self::Image),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" => Some(Self::Numbering),
            "http://schemas.microsoft.com/office/2011/relationships/people" => Some(Self::People),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" => Some(Self::Styles),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" => Some(Self::Settings),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" => Some(Self::Theme),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings" => Some(Self::WebSettings),
            _ => {
                #[cfg(debug_assertions)]
                println!("[Relationships] Unknown relationship type: {}", name);
                Some(Self::Unknown)
            }
        }
    }
}

pub struct Relationship {
    pub id: Rc<str>,
    pub relation_type: RelationshipType,
    pub target: String,
    pub data: Vec<u8>,
}

impl core::fmt::Debug for Relationship {

    /// Custom formatter to avoid dumping the data property.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Relationship")
            .field("id", &self.id)
            .field("relation_type", &self.relation_type)
            .field("target", &self.target)
            .field("data_length", &self.data.len())
            .finish()
    }

}

pub struct Relationships {
    relationships: HashMap<Rc<str>, Rc<RefCell<Relationship>>>
}

impl Relationships {
    pub fn empty() -> Self {
        Self {
            relationships: HashMap::new()
        }
    }

    pub fn load_xml(document: &xml::Document, zip_archive: &mut ZipArchive<File>) -> Result<Self, Error> {
        assert_eq!(document.root_element().tag_name().name(), "Relationships");

        let mut relationships = HashMap::new();

        for relationship_xml in document.root_element().children() {
            if relationship_xml.tag_name().name() != "Relationship" {
                continue;
            }

            #[cfg(feature = "debug-relationships")]
            {
               println!("Relationship");
                for attr in relationship_xml.attributes() {
                    println!("- Attribute \"{}\"  =>  \"{}\"      ns={}", attr.name(), attr.value(), attr.namespace().unwrap_or(""));
                }
            }

            let relation_type = relationship_xml.attribute("Type");
            let relation_type = RelationshipType::convert(relation_type.unwrap());

            let id: Rc<str> = relationship_xml.attribute("Id").unwrap().into();
            let target = relationship_xml.attribute("Target").unwrap();

            let mut data = Vec::new();
            if relation_type.unwrap() == RelationshipType::Image {
                match &mut zip_archive.by_name(&format!("word/{}", target)) {
                    Ok(file) => {
                        std::io::copy(file, &mut data).expect("Failed to read Image");
                    }
                    Err(e) => panic!("Failed to load target \"{}\": {}", target, e)
                }
            }

            relationships.insert(id.clone(), Rc::new(RefCell::new(Relationship{
                id: id.clone(),
                relation_type: relation_type.unwrap(),
                target: String::from(target),
                data
            })));
        }

        Ok(Self {
            relationships
        })
    }

    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    pub fn find(&self, name: &str) -> Option<&Rc<RefCell<Relationship>>> {
        self.relationships.get(name)
    }
}
