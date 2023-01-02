// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod layout;
pub mod numbering;
pub mod painter;

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
    relationships::Relationship
};

use self::painter::Painter;

#[derive(Debug)]
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
    TextRun(),
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
    pub page: usize,

    /// The position this node is starting from.
    pub position: Vector2f,

    pub text_settings: TextSettings,

    pub size: Vector2f,

    pub interaction_states: InteractionStates,

}

impl Node {
    pub fn new(data: NodeData) -> Self {
        Self {
            parent: Weak::new(),
            children: Some(vec![]),

            data,
            page: 0,
            position: Default::default(),
            text_settings: TextSettings::new(),
            size: Default::default(),
            interaction_states: Default::default(),
        }
    }

    /// Run the `callback` function recursively on itself and it's descendants.
    pub fn apply_recursively(&mut self, callback: &dyn Fn(&mut Node)) {
        callback(self);

        if let Some(children) = &mut self.children {
            for child in children {
                callback(&mut child.borrow_mut());
            }
        }
    }

    pub fn on_event(&mut self, event: &mut Event) {
        if let Some(children) = &mut self.children {
            for child in children {
                child.borrow_mut().on_event(event);
            }
        }

        match &self.data {
            NodeData::Hyperlink(hyperlink) => hyperlink.on_event(event),
            NodeData::TextPart(part) => part.on_event(self, event),
            NodeData::Drawing(drawing) => match event {
                Event::Paint(painter) => drawing.draw(self.position, painter),
                _ => ()
            }
            _ => ()
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
                let rect = Rect::new(self.position, self.size);
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

pub fn append_child<'b>(parent_ref: Rc<RefCell<Node>>, mut node: Node) -> Rc<RefCell<Node>> {
    let mut parent = parent_ref.borrow_mut();
    node.parent = Rc::downgrade(&parent_ref);
    node.text_settings = parent.text_settings.clone();
    node.page = parent.page;
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
        page: parent.page,
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
    pub fn new(text_settings: TextSettings, page_settings: PageSettings) -> Node {
        let mut node = Node::new(NodeData::Document(Self {
            page_settings
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

pub enum Event<'a> {
    Click(MouseEvent),
    Hover(MouseEvent),
    Paint(&'a mut Painter<'a>),
}

#[derive(Debug)]
pub struct Paragraph;

#[derive(Debug)]
pub struct Document {
    pub page_settings: PageSettings,
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

#[derive(Debug)]
pub struct StructuredDocumentTag {

}

impl Default for StructuredDocumentTag {
    fn default() -> Self {
        Self {

        }
    }
}
