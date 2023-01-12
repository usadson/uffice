// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{rc::Rc, cell::RefCell};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldType {
    Unknown,

    /// Write the current date.
    Date,

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
            println!("Instruction {}", field_type);
            return match field_type {
                "DATE" => Self {
                    field: FieldType::Date
                },

                "TITLE" => Self {
                    field: FieldType::Title
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

    pub fn resolve_to_string(&self, node: &Rc<RefCell<crate::wp::Node>>) -> String {
        match &self.field {
            FieldType::Date => {
                // When no format is specified, the current date is formatted in
                // an implementation-defined manner:
                chrono::prelude::Local::now().format("%d-%m-%Y").to_string()
            }

            FieldType::Title => {
                let document = node.as_ref().borrow().find_document();
                let document = document.as_ref().borrow();
                if let crate::wp::NodeData::Document(document) = &document.data {
                    if let Some(title) = &document.document_properties.title {
                        return title.clone();
                    }
                }

                String::from("Title Missing")
            }

            _ => format!("{:?}", self)
        }
    }
}
