// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod document_properties;
pub mod instructions;
pub mod layout;
pub mod numbering;

use std::{
    rc::Rc,
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
    Table,
    TableRow,
    TableCell,
    Text,
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

#[derive(Debug)]
pub struct Node {
    /// Can be None when this element isn't allowed to have children
    pub children: Vec<Node>,
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
            children: Vec::new(),

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
    pub fn apply_recursively(&mut self, callback: &dyn Fn(&mut Node, usize), depth: usize) {
        callback(self, depth);

        for child in &mut self.children {
            child.apply_recursively(callback, depth + 1);
        }
    }

    /// Run the `callback` function recursively on itself and it's descendants.
    pub fn apply_recursively_mut(&mut self, callback: &mut dyn FnMut(&mut Node, usize), depth: usize) {
        callback(self, depth);

        for child in &mut self.children {
            child.apply_recursively_mut(callback, depth + 1);
        }
    }

    pub fn on_event(&mut self, event: &mut Event) {
        for child in &mut self.children {
            child.on_event(event);
        }

        if let NodeData::Hyperlink(hyperlink) = &self.data {
            hyperlink.on_event(event);
        }
    }

    /// Returns the hit test result.
    ///
    /// If Some, the vector contains the innermost to outermost nodes that were in the hit path.
    pub fn hit_test(&self, position: Position<f32>, callback: &mut dyn FnMut(&Node)) -> bool {
        for child in &self.children {
            if child.hit_test(position, callback) {
                callback(self);
                return true;
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

    pub fn nth_child_mut(&mut self, index: usize) -> &mut Node {
        &mut self.children[index]
    }

    pub fn update_page_last(&mut self) -> usize {
        let mut last_page = self.page_last;
        for child in &mut self.children {
            let child_last_page = child.update_page_last();
            if last_page < child_last_page {
                last_page = child_last_page;
            }
        }

        self.propose_last_page_number(last_page);

        last_page
    }

    pub fn propose_last_page_number(&mut self, last_page: usize) {
        if self.page_last < last_page {
            self.page_last = last_page;
        }
    }

    pub fn check_last_page_number_from_new_child(&mut self) {
        let mut last_page = self.page_last;

        if let Some(last) = self.children.last() {
            last_page = last.page_last;
        }

        self.propose_last_page_number(last_page);
    }
}

pub fn append_child(parent: &mut Node, mut node: Node) -> usize {
    node.text_settings = parent.text_settings.clone();
    node.page_first = parent.page_last;
    node.page_last = parent.page_last;
    node.position = parent.position;

    parent.children.push(node);
    parent.children.len() - 1
}

pub fn create_child<'b>(parent: &mut Node, data: NodeData) -> usize {
    let node = Node {
        children: Vec::new(),
        data,
        page_first: parent.page_last,
        page_last: parent.page_last,
        position: parent.position,
        text_settings: parent.text_settings.clone(),
        size: Default::default(),
        interaction_states: Default::default(),
    };

    append_child(parent, node)
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

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StructuredDocumentTagLevel {
    /// The tag is found in the document <w:body>.
    Block,

    #[allow(dead_code)]
    Row,

    /// The tag is found in a <w:p> element.
    Inline,

    #[allow(dead_code)]
    Cell,
}

#[derive(Debug)]
pub struct StructuredDocumentTag {
    pub level: StructuredDocumentTagLevel,
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
