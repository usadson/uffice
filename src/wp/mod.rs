// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod document_properties;
pub mod instructions;
pub mod layout;
pub mod numbering;

use std::{
    rc::{Rc, Weak},
    cell::RefCell,
};

use sfml::{system::Vector2f, window::CursorType};

use crate::{
    text_settings::{
        TextSettings,
        PageSettings, Position, Rect
    },
    relationships::Relationship, gui::Size
};

#[derive(Debug, strum_macros::IntoStaticStr)]
pub enum NodeData {
    Document(Document),
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
        if let NodeData::Document(..) = &self {
            return true;
        }

        false
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

#[derive(Debug)]
pub struct Node {

    pub parent: Weak<RefCell<Node>>,

    /// Can be None when this element isn't allowed to have children
    pub children: Option<Vec<Rc<RefCell<Node>>>>,
    pub data: NodeData,

    /// The page number this node is starting on.
    /// (from 0)
    pub page_first: usize,
    pub page_last: usize,

    /// The position this node is starting from.
    pub position: Vector2f,

    pub text_settings: TextSettings,

    pub size: Size<f32>,

    pub interaction_states: InteractionStates,

}

impl Node {
    pub fn new(data: NodeData) -> Self {
        Self {
            parent: Weak::new(),
            children: Some(vec![]),

            data,
            page_first: 0,
            page_last: 0,
            position: Default::default(),
            text_settings: TextSettings::new(),
            size: Default::default(),
            interaction_states: Default::default(),
        }
    }

    /// Run the `callback` function recursively on itself and it's descendants.
    pub fn apply_recursively(&mut self, callback: &dyn Fn(&mut Node, usize), depth: usize) {
        callback(self, depth);

        if let Some(children) = &mut self.children {
            for child in children {
                child.borrow_mut().apply_recursively(callback, depth + 1);
            }
        }
    }

    /// Run the `callback` function recursively on itself and it's descendants.
    pub fn apply_recursively_mut(&mut self, callback: &mut dyn FnMut(&mut Node, usize), depth: usize) {
        callback(self, depth);

        if let Some(children) = &mut self.children {
            for child in children {
                child.borrow_mut().apply_recursively_mut(callback, depth + 1);
            }
        }
    }

    pub fn find_document(&self) -> Rc<RefCell<Node>> {
        if let NodeData::Document(..) = &self.data {
            panic!("find_document whilst we are a document!");
        }

        if let Some(parent) = self.parent.upgrade() {
            let parent_ref = parent.clone();
            let parent_ref = parent_ref.as_ref().borrow();
            let is_document = parent_ref.data.is_document();
            drop(parent_ref);

            if is_document {
                return parent;
            }
        }

        panic!("No document found in tree");
    }

    pub fn on_event(&mut self, event: &mut Event) {
        if let Some(children) = &mut self.children {
            for child in children {
                child.borrow_mut().on_event(event);
            }
        }

        if let NodeData::Hyperlink(hyperlink) = &self.data {
            hyperlink.on_event(event);
        }
    }

    /// Returns the hit test result.
    ///
    /// If Some, the vector contains the innermost to outermost nodes that were in the hit path.
    pub fn hit_test(&mut self, position: Position, callback: &mut dyn FnMut(&mut Node)) -> bool {
        if let Some(children) = &mut self.children {
            for child in children {
                if child.borrow_mut().hit_test(position, callback) {
                    callback(self);
                    return true;
                }
            }
        }

        match self.data {
            NodeData::TextPart(..) => {
                let rect = Rect::new(self.position, Vector2f::new(self.size.width(), self.size.height()));
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
    pub fn set_last_page_number(&mut self, page_number: usize) {
        assert!(self.page_last <= page_number);
        self.page_last = page_number;

        if let Some(parent) = self.parent.upgrade() {
            parent.borrow_mut().set_last_page_number(page_number);
        } else {
            assert!(matches!(self.data, NodeData::Document(..)));
        }
    }
}

pub fn append_child<'b>(parent_ref: Rc<RefCell<Node>>, mut node: Node) -> Rc<RefCell<Node>> {
    let mut parent = parent_ref.borrow_mut();
    node.parent = Rc::downgrade(&parent_ref);
    node.text_settings = parent.text_settings.clone();
    node.page_first = parent.page_last;
    node.page_last = parent.page_last;
    node.position = parent.position;

    if let Some(children) = &mut parent.children {
        children.push(Rc::new(RefCell::new(node)));
        return children.last_mut().unwrap().clone();
    }

    panic!("Node isn't allowed to have children: {:?}", parent.data);
}

pub fn create_child(parent_ref: Rc<RefCell<Node>>, data: NodeData) -> Rc<RefCell<Node>> {
    let mut parent = parent_ref.borrow_mut();
    let node = Node {
        parent: Rc::downgrade(&parent_ref),
        children: Some(vec![]),
        data,
        page_first: parent.page_last,
        page_last: parent.page_last,
        position: parent.position,
        text_settings: parent.text_settings.clone(),
        size: Default::default(),
        interaction_states: Default::default(),
    };

    if let Some(children) = &mut parent.children {
        children.push(Rc::new(RefCell::new(node)));
        return children.last_mut().unwrap().clone();
    }

    panic!("Node isn't allowed to have children: {:?}", parent.data);
}

impl Document {
    pub fn new(text_settings: TextSettings, page_settings: PageSettings,
               document_properties: document_properties::DocumentProperties) -> Node {
        let mut node = Node::new(NodeData::Document(Self {
            page_settings,
            document_properties
        }));

        node.text_settings = text_settings;

        node
    }
}

pub struct MouseEvent {
    pub position: Position,
    pub new_cursor: Option<CursorType>
}

impl MouseEvent {
    pub fn new(position: Position) -> MouseEvent {
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
    pub page_settings: PageSettings,
    pub document_properties: document_properties::DocumentProperties,
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
                mouse_event.new_cursor = Some(CursorType::Hand);
            }

            _ => ()
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
