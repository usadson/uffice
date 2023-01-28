// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::marker::PhantomData;

use super::{painter::{Painter, FontSpecification}, Size, Position, Rect, Color, Brush};

const TAB_MAX_WIDTH: f32 = 220.0;
const TAB_PADDING: f32 = 2.0;

pub trait Widget {
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

    pub fn paint<Iter>(&mut self, painter: &mut dyn Painter, items: Iter)
            where Iter: Iterator<Item = &'a TabItem> {
        painter.paint_rect(Brush::SolidColor(Color::from_rgb(0x17, 0x17, 0x17)), self.bar_rect);

        let tab_brush = Brush::SolidColor(Color::from_rgb(0x80, 0x80, 0x80));
        let mut position = self.bar_rect.position();
        let size = Size::new(TAB_MAX_WIDTH, self.bar_rect.height() - TAB_PADDING * 2.0);

        let tab_font = FontSpecification::new("Segoe UI", 12.0, super::painter::FontWeight::SemiBold);
        painter.select_font(tab_font).unwrap();

        for item in items {
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

            position.x += size.width + TAB_PADDING * 2.0;
        }
    }

}

impl<'a, TabItem> Widget for TabWidget<TabItem>
        where TabItem: TabWidgetItem + 'a {
    fn on_window_resize(&mut self, window_size: Size<u32>) {
        self.bar_rect = Rect::from_position_and_size(
            Position::new(0.0, 0.0),
            Size::new(
                window_size.width as _,
                25.0
            )
        );
    }
}
