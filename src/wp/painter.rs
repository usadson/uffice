// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::{graphics::{RenderTarget, RectangleShape, Color, Transformable, Shape}, system::Vector2f};

use crate::{text_settings::TextSettings, fonts::FontManager};

pub struct Painter<'a> {
    pub render_texture: &'a mut sfml::graphics::RenderTexture,
    pub font_manager: &'a mut FontManager,
}

impl<'a> Painter<'a> {

    pub fn paint_text(&mut self, string: &str, position: Vector2f, settings: &TextSettings) {
        let font = self.font_manager.load_font(settings);
        let text = &mut settings.create_text(&font);

        text.set_string(string);
        text.set_position(position);

        if let Some(highlight_color) = settings.highlight_color {
            self.paint_text_highlight(text, highlight_color);
        }

        // match &mut context.collection_rects {
        //     Some(rects) => {
        //         rects.push(text.global_bounds().into());
        //     }
        //     _ => ()
        // }

        self.render_texture.draw(text);
    }

    fn paint_text_highlight(&mut self, text: &mut sfml::graphics::Text, highlight_color: Color) {
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

        self.render_texture.draw(&shape);
    }


}
