// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod painter;

use sfml::system::Vector2f;

use crate::text_settings::{TextSettings, PageSettings};

use self::painter::Painter;

#[derive(Debug)]
pub enum NodeData {
    Document(Document),
    Hyperlink(),
    Paragraph(Paragraph),
    Text(),
    TextPart(TextPart),
    TextRun(),
}

#[derive(Debug)]
pub struct Node {
    
    pub data: NodeData,

    /// The page number this node is starting on.
    /// (from 0)
    pub page: usize,
    
    /// The position this node is starting from.
    pub position: Vector2f,

    pub text_settings: TextSettings,

    pub size: Vector2f,

    /// Can be None when this element isn't allowed to have children
    pub children: Option<Vec<Node>>,

}

impl Node {
    pub fn append_child(&mut self, node: Node) -> &mut Node {
        if let Some(children) = &mut self.children {
            children.push(node);
            return children.last_mut().unwrap();
        }
        
        panic!("Node isn't allowed to have children: {:?}", self.data);
    }

    pub fn on_event(&mut self, event: &mut Event) {
        if let Some(children) = &mut self.children {
            for child in children {
                child.on_event(event);
            }
        }

        match &self.data {
            NodeData::TextPart(part) => part.on_event(self, event),
            _ => ()
        }
    }
}

impl Document {
    pub fn new(text_settings: TextSettings, page_settings: PageSettings) -> Node {
        Node {
            data: NodeData::Document(Document { 
                page_settings,
            }),
            page: 0, 
            position: Vector2f::new(0.0, 0.0),
            text_settings, 
            size: Vector2f::new(0.0, 0.0),
            children: Some(vec![]),
        }
    }
}

pub enum Event<'a> {
    Paint(&'a mut Painter<'a>)
}

#[derive(Debug)]
pub struct Paragraph;

#[derive(Debug)]
pub struct Document {
    pub page_settings: PageSettings
}

#[derive(Debug)]
pub struct TextPart {
    pub text: String,
}

impl TextPart {
    pub fn on_event(&self, node: &Node, event: &mut Event) {
        match event {
            Event::Paint(painter) => {
                painter.paint_text(&self.text, node.position, &node.text_settings)
            }
            _ => ()
        }
    }
}
