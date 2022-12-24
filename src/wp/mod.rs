// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod painter;

use std::{
    rc::Rc, 
    cell::RefCell
};

use sfml::system::Vector2f;

use crate::{
    text_settings::{
        TextSettings, 
        PageSettings, Position, Rect
    }, 
    relationships::Relationship
};

use self::painter::Painter;

#[derive(Debug)]
pub enum NodeData {
    Document(Document),
    Hyperlink(Hyperlink),
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
            NodeData::Hyperlink(hyperlink) => hyperlink.on_event(self, event),
            NodeData::TextPart(part) => part.on_event(self, event),
            _ => ()
        }
    }

    /// Returns the hit test result.
    /// 
    /// If Some, the vector contains the innermost to outermost nodes that were in the hit path.
    pub fn hit_test(&mut self, position: Position, callback: &dyn Fn(&mut Node)) -> bool {
        if let Some(children) = &mut self.children {
            for child in children {
                if child.hit_test(position, callback) {
                    callback(self);
                    return true;
                }
            }
        }

        match self.data {
            NodeData::TextPart(..) => {
                let rect = Rect::new(self.position, self.size);
                println!("Hit testing rect {:?}", rect);
                if rect.is_inside_inclusive(position) {
                    callback(self);
                    return true;
                }
            }
            _ => ()
        }

        false
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
    Paint(&'a mut Painter<'a>),
    Click(Position),
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

#[derive(Debug)]
pub struct Hyperlink {
    pub relationship: Option<Rc<RefCell<Relationship>>>,
}

impl Hyperlink {
    pub fn on_event(&self, node: &Node, event: &mut Event) {
        if let Event::Click(..) = event {
            if let Some(relationship) = &self.relationship {
                let url = &relationship.borrow().target;
                match url::Url::parse(url) {
                    Err(e) => println!("[Interactable] (Link): \"{}\": {:?}", url, e),
                    Ok(url) => self.open_browser(&url)
                }
            } else {
                println!("[WARNING] Clicked on a link but no relationship was bound :(");
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn open_browser(&self, url: &url::Url) {
        use std::process::Command;
        _ = Command::new("cmd.exe")
                .arg("/C")
                .arg("start")
                .arg("")
                .arg(&url.to_string())
                .spawn();
    }

    #[cfg(target_os = "macos")]
    pub fn open_browser(&self, url: &url::Url) {
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", 
              target_os = "dragonfly", target_os = "netbsd"))]
    pub fn open_browser(&self, url: &url::Url) {

    }
}

impl Default for Hyperlink {
    fn default() -> Self {
        Self {
            relationship: None
        }
    }
}
