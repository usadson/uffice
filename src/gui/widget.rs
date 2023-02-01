// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::marker::PhantomData;

use winit::event::{MouseButton, ElementState};

use super::{
    painter::{Painter, FontSpecification},
    Brush,
    Color,
    MouseMoveEvent,
    Position,
    Rect,
    Size,
};

const TAB_MAX_WIDTH: f32 = 220.0;
const TAB_PADDING: f32 = 6.0;

pub trait Widget {
    fn rect(&self) -> Rect<f32>;
    fn on_mouse_enter(&mut self, event: &mut MouseMoveEvent);
    fn on_mouse_input(&mut self, mouse_position: Position<f32>, button: MouseButton, state: ElementState);
    fn on_mouse_leave(&mut self, event: &mut MouseMoveEvent);
    fn on_mouse_move(&mut self, event: &mut MouseMoveEvent);

    fn on_window_focus_lost(&mut self);

    /// There is no `EventVisualReaction`, because the contents of the
    /// window are always redrawn when the window is resized.
    fn on_window_resize(&mut self, window_size: Size<u32>);
}

pub trait TabWidgetItem {

    /// Retrieves the title of the item, which is displayed to the user as the
    /// text of the tab item.
    fn title(&self) -> String;

}

#[derive(Debug)]
pub struct TabWidget<TabItem>
        where TabItem: TabWidgetItem {
    _marker: PhantomData<TabItem>,
    bar_rect: Rect<f32>,
}

impl<'a, TabItem> TabWidget<TabItem>
        where TabItem: TabWidgetItem + 'a {

    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
            bar_rect: Default::default(),
        }
    }

    pub fn paint<Iter>(&mut self, painter: &mut dyn Painter, items: Iter, selected_nth: Option<usize>)
            where Iter: Iterator<Item = &'a TabItem> {
        painter.paint_rect(Brush::SolidColor(Color::from_rgb(0x80, 0x80, 0x80)), self.bar_rect);

        let tab_brush_normal = Brush::SolidColor(Color::from_rgb(0x45, 0x45, 0x45));
        let tab_brush_selected = Brush::SolidColor(Color::from_rgb(0x1F, 0x1F, 0x1F));
        let mut position = self.bar_rect.position();
        let size = Size::new(TAB_MAX_WIDTH, self.bar_rect.height() - TAB_PADDING * 2.0);

        let tab_font = FontSpecification::new("Segoe UI", 12.0, super::painter::FontWeight::SemiBold);
        painter.select_font(tab_font).unwrap();

        let mut index = 0;
        for item in items {
            let index = {
                let result = index;
                index += 1;
                result
            };

            let is_selected = selected_nth == Some(index);
            let tab_brush = {
                if is_selected {
                    tab_brush_selected
                } else {
                    tab_brush_normal
                }
            };

            let title = item.title();
            let title_text_size = painter.paint_text(Brush::SolidColor(Color::TRANSPARENT), position, &title, None);

            let size = if title_text_size.width + TAB_PADDING * 2.0 < size.width {
                title_text_size
            } else {
                size
            };

            position.x += TAB_PADDING;
            let mut rect = Rect::from_position_and_size(
                Position::new(position.x, position.y + TAB_PADDING),
                size
            );
            painter.paint_rect(tab_brush, rect);


            rect.left += TAB_PADDING;
            rect.right -= TAB_PADDING;
            painter.begin_clip_region(rect);
            painter.paint_text(Brush::SolidColor(Color::WHITE), rect.position(), &title, None);
            painter.end_clip_region();

            position.x += size.width + TAB_PADDING;
        }
    }

}

impl<'a, TabItem> Widget for TabWidget<TabItem>
        where TabItem: TabWidgetItem + 'a {
    fn rect(&self) -> Rect<f32> {
        self.bar_rect
    }

    fn on_mouse_enter(&mut self, _event: &mut MouseMoveEvent) {

    }

    fn on_mouse_input(&mut self, _mouse_position: Position<f32>, _button: MouseButton, _state: ElementState) {

    }

    fn on_mouse_leave(&mut self, _event: &mut MouseMoveEvent) {

    }

    fn on_mouse_move(&mut self, _event: &mut MouseMoveEvent) {

    }

    fn on_window_focus_lost(&mut self) {

    }

    fn on_window_resize(&mut self, window_size: Size<u32>) {
        self.bar_rect = Rect::from_position_and_size(
            Position::new(0.0, 0.0),
            Size::new(
                window_size.width as _,
                33.0
            )
        );
    }
}
