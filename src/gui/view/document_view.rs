// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use roxmltree as xml;

use uffice_lib::{profiling::Profiler, profile_expr};
use winit::window::CursorIcon;

use crate::{
    wp::{
        self,
        numbering::NumberingManager,
        Document,
        Node,
    },
    word_processing::{
        DocumentResult,
        self, HALF_POINT,
    },
    application::load_archive_file_to_string,
    relationships::Relationships,
    style::StyleManager,
    gui::{painter::{FontSpecification, TextCalculator}, Rect, Size, Position},
};

use super::{
    ViewData,
    ViewImpl,
};

/// The height from the top to the first page, and from the end of the
/// last page to the bottom.
pub const VERTICAL_PAGE_MARGIN: f32 = 20.0;

/// The gaps between the pages.
pub const VERTICAL_PAGE_GAP: f32 = 30.0;

#[derive(Debug)]
pub struct DocumentView {
    #[allow(dead_code)]
    view_data: ViewData,

    document: Option<Document>,
    root_node: Option<Node>,

    page_rects: Vec<Rect<f32>>,
}

fn draw_document(archive_path: &str, text_calculator: &mut dyn TextCalculator, progress_sender: &dyn Fn(f32)) -> DocumentResult {
    let mut profiler = Profiler::new(String::from("Document Rendering"));

    let archive_file = profile_expr!(profiler, "Open Archive", std::fs::File::open(archive_path)
            .expect("Failed to open specified file"));

    let mut archive = profile_expr!(profiler, "Read Archive", zip::ZipArchive::new(archive_file)
            .expect("Failed to read ZIP archive"));

    let document_relationships;
    {
        let _frame = profiler.frame(String::from("Document Relationships"));

        let txt = load_archive_file_to_string(&mut archive, "word/_rels/document.xml.rels")
                .expect("Document.xml.rels missing, assuming this is not a DOCX file.");
        if let Ok(document) = xml::Document::parse(&txt) {
            document_relationships = Relationships::load_xml(&document, &mut archive).unwrap();
        } else {
            println!("[Relationships] (word/_rels/document.xml.rels) Error!");
            document_relationships = Relationships::empty();
        }
    }

    let numbering_manager = {
        let _frame = profiler.frame(String::from("Numbering Definitions"));

        if let Some(numbering_document_text) = load_archive_file_to_string(&mut archive, "word/numbering.xml") {
            let numbering_document = xml::Document::parse(&numbering_document_text)
                .expect("Failed to parse numbering document");
            NumberingManager::from_xml(&numbering_document)
        } else {
            NumberingManager::new()
        }
    };

    let style_manager = {
        let _frame = profiler.frame(String::from("Style Definitions"));

        let styles_document_text = load_archive_file_to_string(&mut archive, "word/styles.xml")
                .expect("Style definitions missing, assuming this is not a DOCX file.");
        let styles_document = xml::Document::parse(&styles_document_text)
                .expect("Failed to parse styles document");
        StyleManager::from_document(&styles_document, &numbering_manager).unwrap()
    };

    let mut document_properties = wp::document_properties::DocumentProperties::new();
    if let Some(txt) = load_archive_file_to_string(&mut archive, "docProps/core.xml") {
        if let Ok(document) = xml::Document::parse(&txt) {
            document_properties.import_core_file_properties_part(&document);
        }
    }

    let _frame = profiler.frame(String::from("Document"));
    let document_text = load_archive_file_to_string(&mut archive, "word/document.xml")
            .expect("Archive missing word/document.xml: this file is not a WordprocessingML document!");
    let document = xml::Document::parse(&document_text)
            .expect("Failed to parse document");

    word_processing::process_document(&document, &style_manager, &document_relationships, numbering_manager, document_properties, text_calculator, progress_sender)
}

impl DocumentView {
    pub fn new(archive_path: &str, text_calculator: &mut dyn TextCalculator, progress_sender: &dyn Fn(f32)) -> Self {
        let result = draw_document(archive_path, text_calculator, progress_sender);
        Self {
            view_data: ViewData {  },
            page_rects: Vec::new(),
            document: Some(result.document),
            root_node: Some(result.root_node),
        }
    }

    /// In the future we should construct a layout tree from the DOM tree,
    /// and based on the layout tree a paint tree. That way we can just iterate
    /// the paint nodes and draw the document fast.
    fn paint(&mut self, event: &mut super::PaintEvent) {
        let max_y = event.content_rect.bottom;

        if let Some(document) = &mut self.document {
            let root_node = self.root_node.as_mut().unwrap();

            let page_width = document.page_settings.size.width.get_pts() * event.zoom;
            let page_height = document.page_settings.size.height.get_pts() * event.zoom;
            let page_size = Size::new(page_width, page_height);
            let start_x = event.content_rect.left + (event.content_rect.width() as f32 - page_width) / 2.0;

            self.page_rects.clear();
            let start_y_pages = (root_node.page_first..(root_node.page_last + 1)).map(|index| {
                let page_size_and_margin = VERTICAL_PAGE_GAP + document.page_settings.size.height().get_pts() * event.zoom;
                let start_y = event.content_rect.top + event.start_y + VERTICAL_PAGE_MARGIN * event.zoom + index as f32 * page_size_and_margin;

                if start_y < max_y {
                    event.painter.paint_rect(crate::gui::Brush::SolidColor(crate::gui::Color::WHITE), crate::gui::Rect {
                        left: start_x,
                        right: start_x + page_width,

                        top: start_y,
                        bottom: start_y + page_height
                    });
                }

                self.page_rects.push(Rect::from_position_and_size(Position::new(start_x, start_y), page_size));

                start_y
            }).collect::<Vec<f32>>();

            let mut previous_page = None;

            root_node.apply_recursively_mut(&mut |node, _depth| {
                let start_y = start_y_pages[node.page_first];

                if start_y > max_y {
                    // Outside the bounds of the window.
                    return;
                }

                let position = crate::gui::Position::new(
                    start_x + node.position.x * event.zoom,
                    start_y + node.position.y * event.zoom
                );

                if Some(node.page_first) != previous_page {
                    if previous_page.is_some() {
                        event.painter.end_clip_region();
                    }

                    previous_page = Some(node.page_first);
                    event.painter.begin_clip_region(Rect::from_position_and_size(position, page_size));
                }

                match &node.data {
                    wp::NodeData::TextPart(part) => {
                        let text_size = node.text_settings.non_complex_text_size.unwrap() as f32 * HALF_POINT * event.zoom;
                        let font_family_name = node.text_settings.font.clone().unwrap_or(String::from("Calibri"));
                        event.painter.select_font(FontSpecification::new(&font_family_name, text_size, node.text_settings.font_weight())).unwrap();

                        //let size =
                        event.painter.paint_text(node.text_settings.brush(), position, &part.text, Some(node.size * event.zoom));
                        //println!("Text \"{}\" for size {} and dims {:?}", part.text, text_size, size);
                    }
                    _ => ()
                }
            }, 0);

            if previous_page.is_some() {
                event.painter.end_clip_region();
            }
        }
    }

    fn on_mouse_moved(&mut self, mouse_position: Position<f32>, new_cursor: &mut Option<CursorIcon>) {
        self.check_interactable_for_mouse(mouse_position, &mut |node, position| {
            node.interaction_states.hover = wp::HoverState::HoveringOver;

            let mut event = wp::Event::Hover(wp::MouseEvent::new(position));
            node.on_event(&mut event);

            if let wp::Event::Hover(mouse_event) = &event {
                if let Some(cursor) = mouse_event.new_cursor {
                    *new_cursor = Some(cursor);
                }
            }
        });
    }
}

impl super::ViewImpl for DocumentView {
    /// This function is used so the scroller knows how much we're able to
    /// scroll.
    fn calculate_content_height(&self) -> f32 {
        match self.page_rects.last() {
            Some(page_rect) => page_rect.bottom - self.page_rects.first().unwrap().top,
            None => 0.0
        }
    }

    fn check_interactable_for_mouse(&mut self, mouse_position: Position<f32>, callback: &mut dyn FnMut(&mut crate::wp::Node, Position<f32>)) -> bool {
        // TODO: check if the mouse is inside the bounds of a page.

        let mouse_position = Position::new(mouse_position.x, mouse_position.y);

        // for (_, node) in self.document.as_mut().unwrap().node_arena.iter_mut() {
        //     if let NodeData::TextPart(..) = &node.data {
        //         let node_rect = Rect::from_position_and_size(node.position, node.size);

        //         if node_rect.is_inside_inclusive(mouse_position) {
        //             callback(node, mouse_position);
        //             return true;
        //         }
        //     }
        // }

        false
    }

    fn dump_dom_tree(&mut self) {
        let Some(root_node) = self.root_node.as_mut() else {
            println!("🌲: No tree");
            return;
        };

        root_node.apply_recursively(&|node, depth| {
            print!("🌲: {}{:?}", "    ".repeat(depth), node.data);
            print!(" @ ({}, {})", node.position.x, node.position.y,);
            print!(" sized ({}x{})", node.size.width(), node.size.height());

            println!();
        }, 0);
    }

    fn handle_event(&mut self, event: &mut super::Event) {
        match event {
            super::Event::Paint(event) => self.paint(event),
            super::Event::MouseMoved(mouse_position, new_cursor) =>
                self.on_mouse_moved(*mouse_position, *new_cursor),
        }
    }
}
