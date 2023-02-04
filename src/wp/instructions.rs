// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use super::Document;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldType {
    Unknown,

    /// Write the current date.
    Date,

    /// Write the page number of the specified bookmark.
    PageReference(String),

    Reference(String),

    SequentiallyNumber,

    TableOfContents,

    /// Write the document title.
    Title,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// 17.16.1 Syntax
pub struct Field {
    field: FieldType,

    // TODO add switches
}

impl Field {
    pub fn parse(input: &str) -> Self {
        let mut iter = input.split_ascii_whitespace();
        if let Some(field_type) = iter.next() {
            return match field_type {
                "DATE" => Self {
                    field: FieldType::Date
                },

                "PAGEREF" => Self {
                    field: FieldType::PageReference(iter.next().unwrap_or("//INVALID_REFERENCE//").to_string())
                },

                "REF" => Self {
                    field: FieldType::Reference(iter.next().unwrap_or("//INVALID_REFERENCE//").to_string())
                },

                "SEQ" => Self {
                    field: FieldType::SequentiallyNumber
                },

                "TITLE" => Self {
                    field: FieldType::Title
                },

                "TOC" => Self {
                    field: FieldType::TableOfContents
                },

                _ => {
                    println!("[Instructions] Unknown field_type: \"{}\" in instruction \"{}\"", field_type, input);
                    Self {
                        field: FieldType::Unknown
                    }
                },
            }
        }

        println!("[Instructions] Empty instruction: \"{}\"", input);
        Self {
            field: FieldType::Unknown
        }
    }

    pub fn resolve_to_string(&self, document: &mut Document) -> String {
        match &self.field {
            FieldType::Date => {
                // When no format is specified, the current date is formatted in
                // an implementation-defined manner:
                chrono::prelude::Local::now().format("%d-%m-%Y").to_string()
            }

            FieldType::PageReference(..) => {
                // TODO
                String::from("99999")
            }

            FieldType::Title => {
                if let Some(title) = &document.document_properties.title {
                    return title.clone();
                }

                String::from("Title Missing")
            }

            _ => format!("{:?}", self)
        }
    }
}
