// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;

#[derive(Clone, Default, Debug)]
pub struct DocumentProperties {
    pub creator: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

impl DocumentProperties {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn import_core_file_properties_part(&mut self, document: &xml::Document) {
        for child in document.root_element().children() {
            match child.tag_name().name() {
                "creator" => {
                    self.creator = Some(String::new());
                    for child in child.children() {
                        if child.is_text() && child.text().is_some() {
                            self.creator = Some(String::from(child.text().unwrap()));
                        }
                    }
                }

                "description" => {
                    self.description = Some(String::new());
                    for child in child.children() {
                        if child.is_text() && child.text().is_some() {
                            self.description = Some(String::from(child.text().unwrap()));
                        }
                    }
                }

                "title" => {
                    self.title = Some(String::new());
                    for child in child.children() {
                        if child.is_text() && child.text().is_some() {
                            self.title = Some(String::from(child.text().unwrap()));
                        }
                    }
                }

                _ => ()
            }
        }
    }
}
