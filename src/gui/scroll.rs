// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use winit::window::Window;

use crate::user_settings::{SettingChangeSubscriber, SettingChangeNotification, SettingName};

use super::{
    animate::{EasingFunction, InterpolatedValue},
    painter::Painter,
    Brush,
    Color,
    Position,
    Size,
    Rect,
    InteractionState
};

pub const SCROLL_BAR_WIDTH: f32 = 20.0;

/// The color of the scroll bar below the scroll thumb.
const SCROLL_BAR_BACKGROUND_COLOR: Color = Color::from_rgb(0xBD, 0xBD, 0xBD);

/// The color of the thumb of the scrollbar when it's neither hovered nor
/// clicked.
const SCROLL_BAR_THUMB_DEFAULT_COLOR: Color = Color::from_rgb(0x67, 0x3A, 0xB7);

/// The color of the thumb of the scrollbar when it's hovered over.
const SCROLL_BAR_THUMB_HOVER_COLOR: Color = Color::from_rgb(0x65, 0x32, 0xBC);

/// The color of the thumb of the scrollbar when it's being clicked on.
const SCROLL_BAR_THUMB_CLICK_COLOR: Color = Color::from_rgb(0x60, 0x2B, 0xBC);

const LINE_SPEED: f32 = 100.0;

/// The scroller is responsible for processing the user input (mouse scrolling,
/// thumb dragging), provides a way to calculate a thumb position and size.
pub struct Scroller {
    value: InterpolatedValue,
    pub content_height: f32,
    window_height: f32,

    pub bar_rect: Rect<f32>,
    pub thumb_rect: Rect<f32>,

    pub interaction_state: InteractionState,
}

impl Scroller {
    const EASING_FUNC: EasingFunction = EasingFunction::EaseOutQuadratic;

    /// Instantiates a new scroller.
    pub fn new() -> Self {
        Self {
            value: InterpolatedValue::new(0.0, 150.0, Self::EASING_FUNC, 0.0..1.0),
            content_height: 0.0,
            window_height: 0.0,

            bar_rect: Rect::empty(),
            thumb_rect: Rect::empty(),

            interaction_state: InteractionState::Default,
        }
    }

    /// Scroll the amount of lines specified by the `value` parameter.
    /// Returns whether or not the scroller has scrolled.
    pub fn scroll_lines(&mut self, value: f32) -> bool {
        self.value.increase(-value / self.content_height * LINE_SPEED)
    }

    /// Draws the scroll bar track with the thumb.
    /// TODO: add thumb arrows.
    pub fn paint(&mut self, window: &mut Window, painter: &mut dyn Painter) {
        let window_size = window.inner_size().to_logical::<f32>(window.scale_factor());
        self.window_height = window_size.height;

        let full_page_scrolls = self.content_height / window_size.height as f32;
        let scroll_bar_height = (window_size.height as f32 / full_page_scrolls).ceil();
        let scroll_y = (window_size.height as f32 - scroll_bar_height) * self.value.get();

        let bar_rect = super::Rect::from_position_and_size(
            Position::new(window_size.width - SCROLL_BAR_WIDTH, 0.0),
            Size::new(SCROLL_BAR_WIDTH, window_size.height)
        );
        painter.paint_rect(Brush::SolidColor(SCROLL_BAR_BACKGROUND_COLOR), bar_rect);

        let thumb_color = match self.interaction_state {
            InteractionState::Default => SCROLL_BAR_THUMB_DEFAULT_COLOR,
            InteractionState::Hovered => SCROLL_BAR_THUMB_HOVER_COLOR,
            InteractionState::Pressed => SCROLL_BAR_THUMB_CLICK_COLOR,
        };

        let thumb_rect = super::Rect::from_position_and_size(
            Position::new(bar_rect.left, scroll_y),
            Size::new(SCROLL_BAR_WIDTH, scroll_bar_height)
        );
        painter.paint_rect(Brush::SolidColor(thumb_color), thumb_rect);
    }

    #[allow(dead_code)] // TODO
    pub fn apply_mouse_offset(&mut self, value: f32) {
        let speed = self.window_height as f32 - self.thumb_rect.height();
        self.value.increase(value / speed);
    }

    pub fn position(&mut self) -> f32 {
        self.value.get()
    }
}

impl super::animate::Animated for Scroller {
    fn has_running_animation(&self) -> bool {
        // TODO state changes like is_pressed and is_hovered
        self.value.has_running_animation()
    }
}

impl SettingChangeSubscriber for Scroller {
    fn settings_loaded(&mut self, settings: &crate::user_settings::UserSettings) {
        self.value.set_easing_function(
            if settings.setting_enable_animations() {
                Self::EASING_FUNC
            } else {
                EasingFunction::DisabledAnimations
            }
        );
    }

    fn setting_changed(&mut self, notification: &SettingChangeNotification) {
        if notification.setting_name != SettingName::EnableAnimations {
            return;
        }

        self.settings_loaded(notification.settings);
    }
}
