// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod document_properties;
pub mod instructions;
pub mod layout;
pub mod numbering;

use std::{
    rc::{Rc},
    cell::RefCell,
};

use winit::window::CursorIcon;

use crate::{
    text_settings::{
        TextSettings,
        PageSettings,
    },
    gui::{
        Position,
        Size,
        Rect,
    },
    relationships::Relationship,
};

#[derive(Debug, strum_macros::IntoStaticStr)]
pub enum NodeData {
    /// Line, column or page break.
    Break,
    Document,
    Drawing(crate::drawing_ml::DrawingObject),
    Hyperlink(Hyperlink),

    /// The numbering parent is an invisible parent which contains a single
    /// TextPart child.
    ///
    /// The reason this TextPart has a parent and isn't just a child of the
    /// upper Paragraph, is to combine the TextSettings of the NumberingStyle
    /// and the upperlying Paragraph.
    NumberingParent,
    Paragraph(Paragraph),
    StructuredDocumentTag(StructuredDocumentTag),
    Text(),
    TextPart(TextPart),
    TextRun(TextRun),
}

impl NodeData {
    pub fn is_document(&self) -> bool {
        matches!(self, Self::Document)
    }
}

#[derive(Debug)]
pub enum HoverState {
    HoveringOver,
    NotHoveringOn,
}

#[derive(Debug)]
pub struct InteractionStates {
    pub hover: HoverState,
}

impl Default for InteractionStates {
    fn default() -> Self {
        Self {
            hover: HoverState::NotHoveringOn
        }
    }
}

pub type NodeReference = thunderdome::Index;

#[derive(Debug)]
pub struct Node {

    pub parent: Option<NodeReference>,

    /// Can be None when this element isn't allowed to have children
    pub children: Option<Vec<NodeReference>>,
    pub data: NodeData,

    /// The page number this node is starting on.
    /// (from 0)
    pub page_first: usize,
    pub page_last: usize,

    /// The position this node is starting from.
    pub position: Position<f32>,

    pub text_settings: TextSettings,

    pub size: Size<f32>,

    pub interaction_states: InteractionStates,

}

impl Node {
    pub fn new(data: NodeData) -> Self {
        Self {
            parent: None,
            children: Some(vec![]),

            data,
            page_first: 0,
            page_last: 0,
            position: Position::new(0.0, 0.0),
            text_settings: TextSettings::new(),
            size: Default::default(),
            interaction_states: Default::default(),
        }
    }

    /// Run the `callback` function recursively on itself and it's descendants.
    pub fn apply_recursively(&mut self, document: &mut Document, callback: &dyn Fn(&mut Node, usize), depth: usize) {
        callback(self, depth);

        if let Some(children) = &mut self.children {
            for child in children {
                document.node_arena.get_mut(*child).unwrap().apply_recursively(document, callback, depth + 1);
            }
        }
    }

    /// Run the `callback` function recursively on itself and it's descendants.
    pub fn apply_recursively_mut(&mut self, document: &mut Document, callback: &mut dyn FnMut(&mut Node, usize), depth: usize) {
        callback(self, depth);

        if let Some(children) = &mut self.children {
            for child in children {
                document.node_arena.get_mut(*child).unwrap().apply_recursively_mut(document, callback, depth + 1);
            }
        }
    }

    pub fn on_event(&mut self, document: &mut Document, event: &mut Event) {
        if let Some(children) = &mut self.children {
            for child in children {
                document.node_arena.get_mut(*child).unwrap().on_event(document, event);
            }
        }

        if let NodeData::Hyperlink(hyperlink) = &self.data {
            hyperlink.on_event(event);
        }
    }

    /// Returns the hit test result.
    ///
    /// If Some, the vector contains the innermost to outermost nodes that were in the hit path.
    pub fn hit_test(&mut self, document: &mut Document, position: Position<f32>, callback: &mut dyn FnMut(&mut Node)) -> bool {
        if let Some(children) = &mut self.children {
            for child in children {
                if document.node_arena.get_mut(*child).unwrap().hit_test(document, position, callback) {
                    callback(self);
                    return true;
                }
            }
        }

        match self.data {
            NodeData::TextPart(..) => {
                let rect = Rect::from_position_and_size(self.position, Size::new(self.size.width(), self.size.height()));
                if rect.is_inside_inclusive(position) {
                    callback(self);
                    return true;
                }
            }
            _ => ()
        }

        false
    }

    /// Sets the last page number of this Node and all it's parents.
    pub fn set_last_page_number(&mut self, document: &mut Document, page_number: usize) {
        assert!(self.page_last <= page_number);
        self.page_last = page_number;

        if let Some(parent) = self.parent {
            document.node_arena.get_mut(parent).unwrap().set_last_page_number(document, page_number);
        } else {
            assert!(matches!(self.data, NodeData::Document));
        }
    }
}

pub fn append_child<'b>(document: &mut Document, parent: NodeReference, mut node: Node) -> NodeReference {
    node.parent = Some(parent);

    if let Some(parent) = document.node_arena.get(parent) {
        node.text_settings = parent.text_settings.clone();
        node.page_first = parent.page_last;
        node.page_last = parent.page_last;
        node.position = parent.position;
    }

    let node = document.node_arena.insert(node);

    if let Some(parent) = document.node_arena.get_mut(parent) {
        if let Some(children) = &mut parent.children {
            children.push(node);
            return node;
        }

        panic!("Node isn't allowed to have children: {:?}", parent.data);
    } else {
        panic!("Parent reference is invalid: {:?}", parent);
    }

    // document.node_arena.remove(node);
}

pub fn create_child(document: &mut Document, parent_ref: NodeReference, data: NodeData) -> NodeReference {
    let parent = document.node_arena.get(parent_ref).unwrap();
    let node = Node {
        parent: Some(parent_ref),
        children: Some(Vec::new()),
        data,
        page_first: parent.page_last,
        page_last: parent.page_last,
        position: parent.position,
        text_settings: parent.text_settings.clone(),
        size: Default::default(),
        interaction_states: Default::default(),
    };
    drop(parent);

    let node = document.node_arena.insert(node);

    if let Some(parent) = document.node_arena.get_mut(node) {
        if let Some(children) = &mut parent.children {
            children.push(node);
            return children.last_mut().unwrap().clone();
        }
    }

    panic!("Node isn't allowed to have children: {:?}", parent.data);
}

impl Document {
    pub fn new(text_settings: TextSettings) -> Node {
        let mut node = Node::new(NodeData::Document);

        node.text_settings = text_settings;

        node
    }
}

pub struct MouseEvent {
    pub position: Position<f32>,
    pub new_cursor: Option<CursorIcon>
}

impl MouseEvent {
    pub fn new(position: Position<f32>) -> MouseEvent {
        Self {
            position,
            new_cursor: None
        }
    }
}

pub enum Event {
    Click(MouseEvent),
    Hover(MouseEvent),
}

#[derive(Debug)]
pub struct Paragraph;

#[derive(Debug)]
pub struct Document {
    pub node_arena: thunderdome::Arena<Node>,
    pub page_settings: PageSettings,
    pub document_properties: document_properties::DocumentProperties,
}

impl Document {
    pub fn set_last_page_number(&mut self, mut node_ref: NodeReference, page_number: usize) {
        while let Some(node) = self.node_arena.get(node_ref) {
            if node.page_last < page_number {
                node.page_last = page_number;
            }

            if let Some(parent) = node.parent {
                node_ref = parent;
            } else {
                return;
            }
        }
    }
}

#[derive(Debug)]
pub struct TextPart {
    pub text: String,
}

#[derive(Debug, Default)]
pub struct TextRun {
    pub instruction: Option<crate::wp::instructions::Field>,
}

#[derive(Debug, Default)]
pub struct Hyperlink {
    pub relationship: Option<Rc<RefCell<Relationship>>>,
}

impl Hyperlink {
    pub fn on_event(&self, event: &mut Event) {
        match event {
            Event::Click(..) => {
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

            Event::Hover(mouse_event) => {
                mouse_event.new_cursor = Some(CursorIcon::Hand);
            }
        }
    }

    pub fn get_url(&self) -> Option<String> {
        if let Some(relationship) = &self.relationship {
            return Some(relationship.borrow().target.clone());
        }

        None
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

#[derive(Debug, Default)]
pub struct StructuredDocumentTag {

}

#[derive(Debug)]
pub enum BreakType {
    Column,

    Page,

    /// Line break
    TextWrapping,
}

impl BreakType {

    pub fn from_string(string: Option<&str>) -> Self {
        match string {
            None => BreakType::TextWrapping,
            Some(string) => match string {
                "column" => BreakType::Column,
                "page" => BreakType::Page,
                "textWrapping" => BreakType::TextWrapping,
                _ => {
                    println!("[WP] Warning: unknown BreakType for string \"{}\"", string);
                    BreakType::TextWrapping
                }
            }
        }
    }

}
