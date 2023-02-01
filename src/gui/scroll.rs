// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use winit::event::{MouseButton, ElementState};

use crate::user_settings::{SettingChangeSubscriber, SettingChangeNotification, SettingName};

use super::{
    animate::{EasingFunction, InterpolatedValue},
    painter::Painter,
    Brush,
    Color,
    Position,
    Size,
    Rect,
    InteractionState, MouseMoveEvent, EventVisualReaction
};

pub const SCROLL_BAR_WIDTH: f32 = 15.0;

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

    /// TODO find better names for things that are currently called "content"
    /// and "view".
    view_height: f32,
    thumb_height: InterpolatedValue,

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

            view_height: 0.0,
            thumb_height: InterpolatedValue::new(0.0, 150.0, Self::EASING_FUNC, 0.0..f32::MAX),

            bar_rect: Rect::from_position_and_size(Position::new(0.0, 0.0), Size::new(SCROLL_BAR_WIDTH, 0.0)),
            thumb_rect: Rect::<f32>::empty(),

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
    pub fn paint(&mut self, painter: &mut dyn Painter, inner_content_rect: Rect<f32>) {
        self.view_height = inner_content_rect.height();

        // Reflects into how many parts the content rect can be divided, with
        // each part taking up the whole view rect.
        let full_page_scrolls = self.content_height / self.view_height;
        let mut thumb_height = (self.view_height / full_page_scrolls).ceil();
        if thumb_height.is_infinite() {
            thumb_height = self.view_height;
        }

        let thumb_height = if self.thumb_height.get() == 0.0 {
            self.thumb_height.change_immediately(thumb_height);
            thumb_height
        } else {
            self.thumb_height.change(thumb_height);
            self.thumb_height.get()
        };

        self.bar_rect = super::Rect::from_position_and_size(
            Position::new(inner_content_rect.right, inner_content_rect.top),
            Size::new(SCROLL_BAR_WIDTH, inner_content_rect.height())
        );

        let mut scroll_y = self.bar_rect.top + (self.view_height - thumb_height) * self.value.get();
        if scroll_y.is_nan() {
            scroll_y = 0.0;
        }

        self.thumb_rect = super::Rect::from_position_and_size(
            Position::new(self.bar_rect.left, scroll_y),
            Size::new(SCROLL_BAR_WIDTH, thumb_height)
        );

        self.paint_track(painter);
        self.paint_thumb(painter);
    }

    /// The scroll thumb (or handle) is the part that can be dragged to scroll,
    /// indicating where the user is in the content.
    fn paint_thumb(&self, painter: &mut dyn Painter) {
        let thumb_color = match self.interaction_state {
            InteractionState::Default => SCROLL_BAR_THUMB_DEFAULT_COLOR,
            InteractionState::Hovered => SCROLL_BAR_THUMB_HOVER_COLOR,
            InteractionState::Pressed => SCROLL_BAR_THUMB_CLICK_COLOR,
        };

        painter.paint_rect(Brush::SolidColor(thumb_color), self.thumb_rect);
    }

    /// The track contains the scroll thumb.
    fn paint_track(&self, painter: &mut dyn Painter) {
        painter.paint_rect(Brush::SolidColor(SCROLL_BAR_BACKGROUND_COLOR), self.bar_rect);

        // Border
        painter.paint_rect(Brush::SolidColor(Color::from_rgb(0x80, 0x80, 0x80)),
            Rect::from_position_and_size(
                Position::new(self.bar_rect.left - 1.0, self.bar_rect.top),
                Size::new(1.0, self.bar_rect.height())
            )
        );
    }

    pub fn apply_mouse_offset(&mut self, value: f32) {
        let speed = self.view_height as f32 - self.thumb_rect.height();
        self.value.increase(value / speed);
    }

    pub fn position(&mut self) -> f32 {
        self.value.get()
    }

    pub fn on_mouse_input(&mut self, mouse_position: Position<f32>, button: MouseButton, state: ElementState) {
        if button != MouseButton::Left {
            return;
        }

        self.interaction_state = match state {
            ElementState::Pressed => {
                if self.thumb_rect.is_inside_inclusive(mouse_position) {
                    InteractionState::Pressed
                } else {
                    InteractionState::Default
                }
            },
            ElementState::Released => InteractionState::Default,
        };
    }

    pub fn on_mouse_move(&mut self, event: &mut MouseMoveEvent) {
        if self.interaction_state == InteractionState::Pressed {
            self.apply_mouse_offset(event.delta_y);
            event.reaction = EventVisualReaction::ContentUpdated;
        }
    }

    pub fn on_window_focus_lost(&mut self) {
        self.interaction_state = InteractionState::Default;
    }
}

impl super::animate::Animated for Scroller {
    fn has_running_animation(&self) -> bool {
        // TODO state changes like is_pressed and is_hovered
        self.value.has_running_animation() || self.thumb_height.has_running_animation()
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
