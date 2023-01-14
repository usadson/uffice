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
        Rect,
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
        self,
    },
    application::load_archive_file_to_string,
    relationships::Relationships,
    style::StyleManager,
    text_settings::Position,
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

    content_rect: Rect<f32>,
    page_textures: Vec<Rc<RefCell<RenderTexture>>>,
}

fn draw_document(archive_path: &str) -> DocumentResult {
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
    word_processing::process_document(&document, &style_manager, &document_relationships, numbering_manager, document_properties)
}

impl DocumentView {
    pub fn new(archive_path: &str) -> Self {
        let (page_textures, document) = draw_document(archive_path);

        let mut view = Self {
            view_data: ViewData {  },
            content_rect: Default::default(),
            document: Some(document),
            page_textures: page_textures.render_targets
        };

        view.content_rect = view.calculate_content_rect();

        view
    }

    fn calculate_content_rect(&self) -> Rect<f32> {
        let mut width = 0.0;

        for page in &self.page_textures {
            let width_candidate = page.as_ref().borrow().size().x as f32;
            if width < width_candidate {
                width = width_candidate;
            }
        }

        Rect::<f32>::new(0.0, 0.0, width, self.calculate_content_height())
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
    /// Calculates the height of all the pages, plus some vertical margins and
    /// gaps between the pages.
    ///
    /// This function is used so the scroller knows how much we're able to
    /// scroll.
    fn calculate_content_height(&self) -> f32 {
        // Top and bottom gaps above and below the pages.
        let mut height = VERTICAL_PAGE_MARGIN * 2.0;

        // The gaps between the pages.
        height += (self.page_textures.len() - 1) as f32 * VERTICAL_PAGE_GAP;

        for page_tex in &self.page_textures {
            height += page_tex.as_ref().borrow().size().y as f32;
        }

        height
    }

    fn check_interactable_for_mouse(&self, mouse_position: Vector2f, callback: &mut dyn FnMut(&mut crate::wp::Node, crate::text_settings::Position)) -> bool {
        if !self.content_rect.contains(mouse_position) {
            return false;
        }

        let doc = self.document.as_ref().unwrap();
        let mut document = doc.borrow_mut();

        let mouse_position = Position::new(mouse_position.x as u32, mouse_position.y as u32);
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
                    print!(" sized ({}x{})", node.size.x, node.size.y);

                    println!();
                }, 0);
            }
            None => println!("🌲: No tree"),
        }
    }

    fn handle_event(&mut self, event: &mut super::Event) {
        match event {
            super::Event::Draw(draw_event) => self.draw(draw_event),

            super::Event::Paint(_paint) => {
                println!("Print required:");
                self.dump_dom_tree();
            }

            super::Event::MouseMoved(mouse_position, new_cursor) =>
                self.on_mouse_moved(*mouse_position, *new_cursor),
        }
    }
}

