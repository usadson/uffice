// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{rc::Rc, borrow::BorrowMut, cell::RefCell};

use sfml::{graphics::{RenderTarget, RectangleShape, Color, Transformable, Shape, RenderTexture}, system::Vector2f};

use crate::{text_settings::TextSettings, fonts::FontManager};

pub struct PageRenderTargets {
    pub render_targets: Vec<Rc<RefCell<sfml::graphics::RenderTexture>>>,
}

pub struct Painter<'a> {
    pub page_size: Vector2f,
    pub pages: &'a mut PageRenderTargets,
    pub font_manager: &'a mut FontManager,
    pub last_texture_index: Option<usize>,
}

impl<'a> Painter<'a> {

    pub fn get_page(&mut self, index: usize) -> Rc<RefCell<RenderTexture>> {
        if let Some(last_texture_index) = self.last_texture_index {
            if last_texture_index != index {
                self.pages.render_targets[last_texture_index].as_ref().borrow_mut().display();
            }
        }

        if let Some(render_target) = self.pages.render_targets.get(index) {
            return render_target.clone();
        }

        let mut render_texture = RenderTexture::new(self.page_size.x as u32, self.page_size.y as u32)
            .expect(&format!("Failed to create RenderTexture for page #{}", index));

        render_texture.clear(Color::WHITE);

        render_texture.display();
        render_texture.set_smooth(true);

        let render_texture = Rc::new(RefCell::new(render_texture));

        self.pages.render_targets.insert(index, render_texture.clone());
        render_texture
    }

    pub fn paint_text(&mut self, string: &str, page: usize, position: Vector2f, settings: &TextSettings) {
        let font = self.font_manager.load_font(settings);
        let text = &mut settings.create_text(&font);

        text.set_string(string);
        text.set_position(position);

        if let Some(highlight_color) = settings.highlight_color {
            self.paint_text_highlight(page, text, highlight_color);
        }

        // match &mut context.collection_rects {
        //     Some(rects) => {
        //         rects.push(text.global_bounds().into());
        //     }
        //     _ => ()
        // }

        self.get_page(page).as_ref().borrow_mut().draw(text);
    }

    fn paint_text_highlight(&mut self, page: usize, text: &mut sfml::graphics::Text, highlight_color: Color) {
        let mut shape = RectangleShape::new();

        shape.set_position(text.position());

        let size = text.local_bounds().size();
        shape.set_size(Vector2f::new(size.x, text.character_size() as f32 + 30.0));
        shape.set_fill_color(highlight_color);

        // match &mut context.collection_rects {
        //     Some(rects) => {
        //         rects.push(shape.global_bounds().into());
        //     }
        //     _ => ()
        // }

        self.get_page(page).as_ref().borrow_mut().draw(&shape);
    }

    pub fn finish(&mut self) {
        if let Some(last_texture_index) = self.last_texture_index {
            self.pages.render_targets[last_texture_index].as_ref().borrow_mut().display();
        }
        self.last_texture_index = None;
    }
}
