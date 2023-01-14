// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{
    rc::Rc,
    cell::RefCell,
};

use roxmltree as xml;

use sfml::{
    graphics::{
        RenderTexture,
        RenderTarget,
        Sprite,
        Transformable, Color,
    },
    system::Vector2f,
    window::CursorType,
};

use uffice_lib::{profiling::Profiler, profile_expr};

use crate::{
    wp::{
        self,
        numbering::NumberingManager,
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

    document: Option<Rc<RefCell<crate::wp::Node>>>,

    page_rects: Vec<Rect<f32>>,

    content_rect: sfml::graphics::Rect<f32>,
    page_textures: Vec<Rc<RefCell<RenderTexture>>>,
}

fn draw_document(archive_path: &str, text_calculator: &mut dyn TextCalculator) -> DocumentResult {
    let mut profiler = Profiler::new(String::from("Document Rendering"));

    let archive_file = profile_expr!(profiler, "Open Archive", std::fs::File::open(archive_path)
            .expect("Failed to open specified file"));

    let mut archive = profile_expr!(profiler, "Read Archive", zip::ZipArchive::new(archive_file)
            .expect("Failed to read ZIP archive"));

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("[Document] ZIP: File: {}", file.name());
    }

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
    word_processing::process_document(&document, &style_manager, &document_relationships, numbering_manager, document_properties, text_calculator)
}

impl DocumentView {
    pub fn new(archive_path: &str, text_calculator: &mut dyn TextCalculator) -> Self {
        let (page_textures, document) = draw_document(archive_path, text_calculator);

        let mut view = Self {
            view_data: ViewData {  },
            page_rects: Vec::new(),
            content_rect: Default::default(),
            document: Some(document),
            page_textures: page_textures.render_targets
        };

        view.content_rect = view.calculate_content_rect();

        view
    }

    fn calculate_content_rect(&self) -> sfml::graphics::Rect<f32> {
        let mut width = 0.0;

        for page in &self.page_textures {
            let width_candidate = page.as_ref().borrow().size().x as f32;
            if width < width_candidate {
                width = width_candidate;
            }
        }

        sfml::graphics::Rect::<f32>::new(0.0, 0.0, width, self.calculate_content_height())
    }

    fn draw(&mut self, event: &mut super::DrawEvent) {
        let mut y = event.start_y;
        for render_texture in &self.page_textures {
            // I don't know rust well enough to be able to keep a Sprite
            // around _and_ replace the texture.
            //
            // But since this code is not performance-critical I don't care
            // atm.

            let texture = render_texture.borrow();
            let mut sprite = Sprite::with_texture(texture.texture());

            let full_size = event.window_size.x as f32;
            let page_size = sprite.texture_rect().width as f32;

            let scale = full_size * event.zoom / page_size;
            let centered_x = (full_size - page_size * scale) / 2.0;
            sprite.set_scale((scale, scale));

            sprite.set_position((
                centered_x,
                y
            ));

            y += sprite.global_bounds().size().y + VERTICAL_PAGE_GAP * event.zoom;

            sprite.set_color(Color::rgba(255, 255, 255, (255.0 * event.opaqueness) as u8));
            event.window.draw(&sprite);
        }
    }

    /// In the future we should construct a layout tree from the DOM tree,
    /// and based on the layout tree a paint tree. That way we can just iterate
    /// the paint nodes and draw the document fast.
    fn paint(&mut self, event: &mut super::PaintEvent) {
        let max_y = event.window_size.height as f32;

        if let Some(document) = &self.document {
            let (first_page, last_page, page_settings) = {
                let doc = document.as_ref().borrow();

                (doc.page_first, doc.page_last, match &doc.data {
                    crate::wp::NodeData::Document(d) => {
                        d.page_settings
                    }
                    _ => panic!("Invalid document Node: {:?}", document),
                })
            };

            let page_width = page_settings.size.width as f32 * event.zoom / 12.0;
            let page_height = page_settings.size.height as f32 * event.zoom / 12.0;
            let page_size = Size::new(page_width, page_height);
            let start_x = (event.window_size.width as f32 - page_width) / 2.0;

            let start_y_pages = (first_page..(last_page + 1)).map(|index| {
                let page_size_and_margin = (VERTICAL_PAGE_GAP + page_settings.size.height as f32) * event.zoom;
                let start_y = event.start_y + VERTICAL_PAGE_MARGIN * event.zoom + index as f32 * page_size_and_margin;

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

            document.borrow_mut().apply_recursively_mut(&mut |node, _depth| {
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
                        event.painter.paint_text(node.text_settings.brush(), position, &part.text);
                    }
                    _ => ()
                }
            }, 0);

            if previous_page.is_some() {
                event.painter.end_clip_region();
            }
        }
    }

    fn on_mouse_moved(&mut self, mouse_position: Vector2f, new_cursor: &mut Option<CursorType>) {
        let Some(document) = &mut self.document else {
            return;
        };

        document.borrow_mut().apply_recursively(&mut |node, _depth| {
            node.interaction_states.hover = wp::HoverState::NotHoveringOn;
        }, 0);

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
            Some(page_rect) => page_rect.bottom,
            None => 0.0
        }
    }

    fn check_interactable_for_mouse(&self, mouse_position: Vector2f, callback: &mut dyn FnMut(&mut crate::wp::Node, crate::text_settings::Position)) -> bool {
        if !self.content_rect.contains(mouse_position) {
            return false;
        }

        let doc = self.document.as_ref().unwrap();
        let mut document = doc.borrow_mut();

        let mouse_position = crate::text_settings::Position::new(mouse_position.x as u32, mouse_position.y as u32);
        document.hit_test(mouse_position, &mut |node| {
            callback(node, mouse_position);
        })
    }

    fn dump_dom_tree(&self) {
        match &self.document {
            Some(document) => {
                document.borrow_mut().apply_recursively(&|node, depth| {
                    print!("🌲: {}{:?}", "    ".repeat(depth), node.data);
                    print!(" @ ({}, {})", node.position.x, node.position.y,);
                    print!(" sized ({}x{})", node.size.width(), node.size.height());

                    println!();
                }, 0);
            }
            None => println!("🌲: No tree"),
        }
    }

    fn handle_event(&mut self, event: &mut super::Event) {
        match event {
            super::Event::Draw(draw_event) => self.draw(draw_event),
            super::Event::Paint(event) => self.paint(event),
            super::Event::MouseMoved(mouse_position, new_cursor) =>
                self.on_mouse_moved(*mouse_position, *new_cursor),
        }
    }
}
