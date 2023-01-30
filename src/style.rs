// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;
use std::collections::HashMap;

use crate::{error::Error, WORD_PROCESSING_XML_NAMESPACE, text_settings::TextSettings};

struct Style {
    text_settings: TextSettings
}

fn is_correct_namespace(element: &xml::Node) -> bool {
    if element.tag_name().namespace().is_none() {
        return false;
    }

    element.tag_name().namespace().unwrap() == WORD_PROCESSING_XML_NAMESPACE
}

impl Style {

    pub fn from_document_by_style_id(manager: &mut StyleManager, numbering_manager: &crate::wp::numbering::NumberingManager,
                                     document: &xml::Document, name: &str) -> Result<Self, Error> {
        assert!(is_correct_namespace(&document.root_element()));

        for element in document.root_element().children() {
            if !is_correct_namespace(&element) || element.tag_name().name() != "style" {
                continue;
            }

            if let Some(id) = element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")) {
                if id == name {
                    return Self::from_xml(manager, numbering_manager, &element)
                }
            }
        }

        Err(Error::StyleNotFound)
    }

    pub fn from_xml(manager: &mut StyleManager, numbering_manager: &crate::wp::numbering::NumberingManager, element: &xml::Node) -> Result<Self, Error> {
        assert!(element.tag_name().namespace().is_some());
        assert_eq!(element.tag_name().namespace().unwrap(), WORD_PROCESSING_XML_NAMESPACE);

        let mut style = Style{
            text_settings: TextSettings::new()
        };

        for child in element.children() {
            #[cfg(feature = "debug-styles")]
            println!("Style>> {}", child.tag_name().name());

            if child.tag_name().namespace().is_none() || child.tag_name().namespace().unwrap() != WORD_PROCESSING_XML_NAMESPACE {
                println!("Incorrect namespace: {:?}", child.tag_name().namespace());
                continue;
            }

            match child.tag_name().name() {
                "basedOn" => {
                    let val = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                            .expect("No w:val attribute on w:basedOn element!");

                    assert_ne!(element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")).unwrap(), val,
                            "The w:basedOn is used recursively on the same <w:style>! This is an error!");

                    if let Ok(based_on_style) = manager.find_style_using_document(val, element.document(), numbering_manager) {
                        style.inherit_from(based_on_style);
                    }
                }
                "rPr" => {
                    let mut settings = style.text_settings;
                    settings.apply_run_properties_element(manager, &child);
                    style.text_settings = settings;
                }
                "pPr" => {
                    crate::word_processing::process_paragraph_properties_element(numbering_manager, manager,
                        &mut style.text_settings, &child);
                }
                _ => {
                    #[cfg(feature = "debug-styles")]
                    println!("  Unknown");
                }
            }
        }

        Ok(style)
    }

    fn inherit_from(&mut self, style: &Style) {
        self.text_settings = style.text_settings.clone();
    }

}

pub struct StyleManager {
    styles: HashMap<String, Style>,
    default_text_settings: TextSettings,
}

fn process_xml_doc_defaults(element: &xml::Node, manager: &mut StyleManager) {
    for child in element.children() {
        #[cfg(feature = "debug-styles")]
        println!("Style⟫ │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "rPrDefault" => {
                process_xml_rpr_default(&child, manager);
            }
            _ => ()
        }
    }
}

fn process_xml_rpr_default(element: &xml::Node, manager: &mut StyleManager) {
    for child in element.children() {
        #[cfg(feature = "debug-styles")]
        println!("Style⟫ │  │  ├─ {}", child.tag_name().name());

        match child.tag_name().name() {
            "rPr" => {
                let mut settings = manager.default_text_settings.clone();

                settings.apply_run_properties_element(manager, &child);

                manager.default_text_settings = settings;
            }
            _ => ()
        }
    }
}

impl StyleManager {
    pub fn from_document(document: &xml::Document, numbering_manager: &crate::wp::numbering::NumberingManager) -> Result<Self, Error> {
        let mut manager = StyleManager{
            styles: HashMap::new(),
            default_text_settings: TextSettings::new()
        };

        assert_eq!(document.root_element().tag_name().name(), "styles");
        assert!(is_correct_namespace(&document.root_element()));

        #[cfg(feature = "debug-styles")]
        println!("Style⟫ {}", document.root_element().tag_name().name());

        for element in document.root_element().children() {
            #[cfg(feature = "debug-styles")]
            println!("Style⟫ ├─ {}", element.tag_name().name());

            if !is_correct_namespace(&element) {
                continue;
            }

            match element.tag_name().name() {
                "docDefaults" => process_xml_doc_defaults(&element, &mut manager),
                "style" =>
                    match element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")) {
                        Some(id) => {
                            #[cfg(feature = "debug-styles")]
                            println!("Style> {}", id);
                            let style = Style::from_xml(&mut manager, numbering_manager, &element)?;
                            manager.styles.insert(String::from(id), style);
                        }
                        None => {
                            println!("[Styles] Warning: <w:style> doesn't have a w:styleId attribute!");
                        }
                    }
                _ => ()
            }
        }

        Ok(manager)
    }

    fn find_style_using_document(&mut self, name: &str, document: &xml::Document, numbering_manager: &crate::wp::numbering::NumberingManager) -> Result<&Style, Error> {
        if !self.styles.contains_key(name) {
            let style = Style::from_document_by_style_id(self, numbering_manager, document, name)?;

            self.styles.insert(String::from(name), style);
        }

        Ok(self.find_style(name).unwrap())
    }

    fn find_style(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    pub fn apply_paragraph_style(&self, style_id: &str, paragraph_text_settings: &mut TextSettings) {
        if let Some(style) = self.styles.get(style_id) {
            paragraph_text_settings.inherit_from(&style.text_settings);
        } else {
            panic!("Style not found: {}", style_id);
        }
    }

    pub fn apply_character_style(&self, style_id: &str, text_settings: &mut TextSettings) {
        if let Some(style) = self.styles.get(style_id) {
            text_settings.inherit_from(&style.text_settings);
        }
    }

    pub fn default_text_settings(&self) -> TextSettings {
        self.default_text_settings.clone()
    }
}
