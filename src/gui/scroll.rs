// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::{graphics::{RenderWindow, RectangleShape, Shape, RenderTarget, Transformable, Rect, Color}, system::Vector2f};
use uffice_lib::math;

use crate::application::Animator;

pub const SCROLL_BAR_WIDTH: f32 = 20.0;

/// The color of the scroll bar below the scroll thumb.
const SCROLL_BAR_BACKGROUND_COLOR: Color = Color::rgb(0xBD, 0xBD, 0xBD);

/// The color of the thumb of the scrollbar when it's neither hovered nor
/// clicked.
const SCROLL_BAR_THUMB_DEFAULT_COLOR: Color = Color::rgb(0x67, 0x3A, 0xB7);

/// The color of the thumb of the scrollbar when it's hovered over.
const SCROLL_BAR_THUMB_HOVER_COLOR: Color = Color::rgb(0x65, 0x32, 0xBC);

/// The color of the thumb of the scrollbar when it's being clicked on.
const SCROLL_BAR_THUMB_CLICK_COLOR: Color = Color::rgb(0x60, 0x2B, 0xBC);

pub struct Scroller {
    value: f32,
    pub content_height: f32,
    window_height: f32,

    pub bar_rect: Rect<f32>,
    pub thumb_rect: Rect<f32>,

    pub is_hovered: bool,
    pub is_pressed: bool,

    animator: Animator,
    value_increase: f32,
}

impl Scroller {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            content_height: 0.0,
            window_height: 0.0,
            bar_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            thumb_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            is_hovered: false,
            is_pressed: false,
            animator: Animator::new_with_delay(150.0),
            value_increase: 0.0,
        }
    }

    pub fn scroll(&mut self, value: f32) {
        self.increase_thumb_position(-value / 100.0);
    }

    pub fn draw(&mut self, shape: &mut RectangleShape, parent: &mut RenderWindow) {
        let window_size = parent.size();
        self.window_height = window_size.y as f32;

        let full_page_scrolls = self.content_height / window_size.y as f32;
        let scroll_bar_height = (window_size.y as f32 / full_page_scrolls).ceil();
        let scroll_y = (window_size.y as f32 - scroll_bar_height) * Scroller::bound_position(self.value + self.value_increase);

        shape.set_fill_color(SCROLL_BAR_BACKGROUND_COLOR);
        shape.set_size(Vector2f::new(SCROLL_BAR_WIDTH, window_size.y as f32));
        shape.set_position(Vector2f::new(window_size.x as f32 - SCROLL_BAR_WIDTH, 0.0));
        self.bar_rect = shape.global_bounds();
        parent.draw(shape);

        shape.set_fill_color({
            if self.is_pressed {
                SCROLL_BAR_THUMB_CLICK_COLOR
            } else if self.is_hovered {
                SCROLL_BAR_THUMB_HOVER_COLOR
            } else {
                SCROLL_BAR_THUMB_DEFAULT_COLOR
            }
        });
        shape.set_size(Vector2f::new(SCROLL_BAR_WIDTH, scroll_bar_height));
        shape.set_position(Vector2f::new(window_size.x as f32 - SCROLL_BAR_WIDTH, scroll_y));
        self.thumb_rect = shape.global_bounds();
        parent.draw(shape);
    }

    pub fn apply_mouse_offset(&mut self, value: f32) {
        self.increase_thumb_position(value / (self.window_height as f32 - self.thumb_rect.height));
    }

    pub fn increase_thumb_position(&mut self, value: f32) {
        let increase = self.animator.update() * self.value_increase;
        self.set_thumb_position(self.value + increase);
        self.animator.reset();
        self.value_increase += value - increase;
    }

    fn set_thumb_position(&mut self, value: f32) {
        self.value = Scroller::bound_position(value);
    }

    pub fn position(&mut self) -> f32 {
        Scroller::bound_position(self.value + math::lerp_precise_f32(0.0, self.value_increase, self.animator.update()))
    }

    pub fn bound_position(value: f32) -> f32 {
        match value {
            d if d < 0.0 => 0.0,
            d if d > 1.0 => 1.0,
            d => d,
        }
    }
}
