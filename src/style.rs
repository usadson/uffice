// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;
use std::collections::HashMap;

use crate::{error::Error, WORD_PROCESSING_XML_NAMESPACE, text_settings::TextSettings, apply_run_properties_for_paragraph_mark};

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

    pub fn from_document_by_style_id(manager: &mut StyleManager, document: &xml::Document, name: &str) -> Result<Self, Error> {
        assert!(is_correct_namespace(&document.root_element()));

        for element in document.root_element().children() {
            if !is_correct_namespace(&element) || element.tag_name().name() != "style" {
                continue;
            }

            match element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")) {
                Some(id) => {
                    if id == name {
                        return Self::from_xml(manager, &element)
                    }
                }
                None => ()
            }
        }

        Err(Error::StyleNotFound)
    }

    pub fn from_xml(manager: &mut StyleManager, element: &xml::Node) -> Result<Self, Error> {
        assert!(element.tag_name().namespace().is_some());
        assert_eq!(element.tag_name().namespace().unwrap(), WORD_PROCESSING_XML_NAMESPACE);

        let mut style = Style{
            text_settings: TextSettings::new()
        };

        for child in element.children() {
            if child.tag_name().namespace().is_none() || child.tag_name().namespace().unwrap() != WORD_PROCESSING_XML_NAMESPACE {
                continue;
            }

            match child.tag_name().name() {
                "basedOn" => {
                    let val = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                            .expect("No w:val attribute on w:basedOn element!");
                    
                    assert_ne!(element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")).unwrap(), val,
                            "The w:basedOn is used recursively on the same <w:style>! This is an error!");

                    if let Ok(based_on_style) = manager.find_style_using_document(val, element.document()) {
                        style.inherit_from(based_on_style);
                    }
                }
                "rPr" => {
                    let mut settings = style.text_settings;
                    apply_run_properties_for_paragraph_mark(&child, &mut settings);
                    style.text_settings = settings;
                }
                _ => ()
            }
        }
        
        Ok(style)
    } 

    fn inherit_from(self: &mut Self, style: &Style) {
        self.text_settings = style.text_settings.clone();
    }

}

pub struct StyleManager {
    styles: HashMap<String, Style>,
    default_text_settings: TextSettings,
}

fn process_xml_doc_defaults(element: &xml::Node, manager: &mut StyleManager) {
    for child in element.children() {
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
        println!("Style⟫ │  │  ├─ {}", child.tag_name().name());
        match child.tag_name().name() {
            "rPr" => {
                apply_run_properties_for_paragraph_mark(&child, &mut manager.default_text_settings);
            }
            _ => ()
        }
    }
}

impl StyleManager {
    pub fn from_document(document: &xml::Document) -> Result<Self, Error> {
        let mut manager = StyleManager{
            styles: HashMap::new(), 
            default_text_settings: TextSettings::new()
        };

        assert_eq!(document.root_element().tag_name().name(), "styles");
        assert!(is_correct_namespace(&document.root_element()));

        println!("Style⟫ {}", document.root_element().tag_name().name());

        for element in document.root_element().children() {
            println!("Style⟫ ├─ {}", element.tag_name().name());
            if !is_correct_namespace(&element) {
                continue;
            }

            match element.tag_name().name() {
                "docDefaults" => process_xml_doc_defaults(&element, &mut manager),
                "style" =>
                    match element.attribute((WORD_PROCESSING_XML_NAMESPACE, "styleId")) {
                        Some(id) => {
                            let style = Style::from_xml(&mut manager, &element)?;
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

    fn find_style_using_document(self: &mut Self, name: &str, document: &xml::Document) -> Result<&Style, Error> {
        if !self.styles.contains_key(name) {
            let style = Style::from_document_by_style_id(self, document, name)?;

            self.styles.insert(String::from(name), style);
        }

        Ok(self.find_style(name).unwrap())
    }

    fn find_style(self: &Self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    pub fn apply_paragraph_style(&self, style_id: &str, paragraph_text_settings: &mut TextSettings) {
        if let Some(style) = self.styles.get(style_id) {
            paragraph_text_settings.inherit_from(&style.text_settings);
        }
    }

    pub fn default_text_settings(&self) -> &TextSettings {
        &self.default_text_settings
    }
}
