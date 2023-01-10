// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldType {
    Unknown,

    /// Write the current date.
    Date,
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

    pub fn resolve_to_string(&self) -> String {
        match &self.field {
            FieldType::Date => {
                // When no format is specified, the current date is formatted in
                // an implementation-defined manner:
                chrono::prelude::Local::now().format("%d-%m-%Y").to_string()
            }

            _ => format!("{:?}", self)
        }
    }
}
