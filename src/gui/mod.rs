// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use crate::application::TabId;

pub mod animate;
pub mod app;
pub mod painter;
pub mod scroll;
pub mod view;
pub mod widget;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect<T> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T> Rect<T> where T: Copy {
    /// Creates a Rect with no size.
    pub fn empty() -> Rect<T>
            where T: Copy + Default {
        Self {
            left: Default::default(),
            right: Default::default(),
            top: Default::default(),
            bottom: Default::default(),
        }
    }

    pub fn from_positions(left: T, right: T, top: T, bottom: T) -> Self {
        Self { left, right, top, bottom }
    }

    pub fn left(&self) -> T {
        self.left
    }

    pub fn right(&self) -> T {
        self.right
    }

    pub fn top(&self) -> T {
        self.top
    }

    pub fn bottom(&self) -> T {
        self.bottom
    }

    pub fn position(&self) -> Position<T> {
        Position::new(self.left, self.top)
    }
}

impl<T> Rect<T> where T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::cmp::PartialOrd {
    pub fn from_position_and_size(position: Position<T>, size: Size<T>) -> Self {
        Self {
            left: position.x(),
            right: position.x() + size.width(),
            top: position.y(),
            bottom: position.y() + size.height()
        }
    }

    /// Get the width of the Rect.
    pub fn width(&self) -> T {
        self.right - self.left
    }

    /// Get the height of the Rect.
    pub fn height(&self) -> T {
        self.bottom - self.top
    }

    pub fn size(&self) -> Size<T> {
        Size::new(self.width(), self.height())
    }

    pub fn is_inside_inclusive(&self, position: Position<T>) -> bool {
        position.x() >= self.left && position.x() <= self.right
            && position.y() >= self.top && position.y() <= self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test() {
        assert_eq!(Rect::<f32>::empty(), Rect::from_position_and_size(Position::new(0.0, 0.0), Size::new(0.0, 0.0)));

        assert_eq!(Rect::<f32>::empty().bottom, 0.0);
        assert_eq!(Rect::<f32>::empty().top, 0.0);
        assert_eq!(Rect::<f32>::empty().right, 0.0);
        assert_eq!(Rect::<f32>::empty().left, 0.0);

        assert_eq!(Rect::<f32>::empty().width(), 0.0);
        assert_eq!(Rect::<f32>::empty().height(), 0.0);

        assert_eq!(Rect::<f32>::empty().position(), Position::new(0.0, 0.0));
        assert_eq!(Rect::<f32>::empty().size(), Size::<f32>::empty());
    }
}

/// Defines a size. Prefer this over using Vector2f for everything since it
/// communicates the definition better.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size<T> {
    width: T,
    height: T,
}

impl From<Position<f32>> for Size<f32> {
    fn from(value: Position<f32>) -> Self {
        Self {
            width: value.x(),
            height: value.y(),
        }
    }
}

impl From<Size<u32>> for Size<f32> {
    fn from(value: Size<u32>) -> Self {
        Self {
            width: value.width as _,
            height: value.height as _,
        }
    }
}

impl From<Size<f32>> for Size<u32> {
    fn from(value: Size<f32>) -> Self {
        Self {
            width: value.width as _,
            height: value.height as _,
        }
    }
}

impl<T> Size<T> where T: Copy {
    /// Creates a size with no width or height.
    pub fn empty() -> Size<T>
            where T: Copy + Default {
        Self { width: Default::default(), height: Default::default() }
    }

    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }
}

impl std::ops::Mul<f32> for Size<f32> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl<T> From<winit::dpi::LogicalSize<T>> for Size<T> {
    fn from(value: winit::dpi::LogicalSize<T>) -> Self {
        Self { width: value.width, height: value.height }
    }
}

/// Defines a size. Prefer this over using Vector2f for everything since it
/// communicates the definition better.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Position<T> {
    x: T,
    y: T,
}

impl<T> Position<T> where T: Copy {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Get the x value.
    pub fn x(&self) -> T {
        self.x
    }

    /// Get the y value.
    pub fn y(&self) -> T {
        self.y
    }

    /// Get a mutable reference to the x value.
    pub fn x_mut(&mut self) -> &mut T {
        &mut self.x
    }

    /// Get a mutable reference to the y value.
    pub fn y_mut(&mut self) -> &mut T {
        &mut self.y
    }
}

impl<T: std::ops::Mul<T, Output = T> + Copy> std::ops::Mul<T> for Position<T> {
    type Output = Position<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Position::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: std::ops::Add<T, Output = T> + Copy> std::ops::Add<Position<T>> for Position<T> {
    type Output = Position<T>;

    fn add(self, rhs: Position<T>) -> Self::Output {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: std::ops::Sub<T, Output = T> + Copy> std::ops::Sub<Position<T>> for Position<T> {
    type Output = Position<T>;

    fn sub(self, rhs: Position<T>) -> Self::Output {
        Position::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// An RGBA color.
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {

    /// Black, or in hex notation: #000000
    pub const BLACK: Color = Color::from_rgb(0, 0, 0);

    /// White, or in hex notation: #FFFFFF
    pub const WHITE: Color = Color::from_rgb(255, 255, 255);

    /// Red, or in hex notation: #FF0000
    pub const RED: Color = Color::from_rgb(255, 0, 0);

    /// Green, or in hex notation: #00FF00
    pub const GREEN: Color = Color::from_rgb(0, 255, 0);

    /// Blue, or in hex notation: #0000FF
    pub const BLUE: Color = Color::from_rgb(0, 0, 255);

    /// A fully transparent color, or in hex notation: #00000000
    pub const TRANSPARENT: Color = Color::from_rgba(0, 0, 0, 0);

    /// Creates a color from RGB color components, with full alpha opaqueness.
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self  {
        Self { red, green, blue, alpha: 255 }
    }

    /// Creates a color from RGBA color components.
    pub const fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self  {
        Self { red, green, blue, alpha }
    }

    /// Get the red component of the RGBA color.
    pub fn red(&self) -> u8 {
        self.red
    }

    /// Get the green component of the RGBA color.
    pub fn green(&self) -> u8 {
        self.green
    }

    /// Get the blue component of the RGBA color.
    pub fn blue(&self) -> u8 {
        self.blue
    }

    /// Get the alpha component of the RGBA color.
    pub fn alpha(&self) -> u8 {
        self.alpha
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Brush {
    Test,

    /// A brush with a solid color.
    SolidColor(Color),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    PainterRequest,

    /// A certain tab was loading and is now ready.
    TabBecameReady(TabId),

    /// A certain tab was painted.
    TabPainted {
        tab_id: TabId,

        /// The total height of the content in pixels.
        total_content_height: f32,
    },

    /// A certain tab has progressed in loading.
    TabProgressed {
        tab_id: TabId,

        /// The progress of the tab, between 0 and 1.
        progress: f32,
    },

    TabCrashed {
        tab_id: TabId,
    },

}

unsafe impl Send for AppEvent {}

/// Defines the state a component is in. This ensures the correct animations and
/// subsequent state. This state doesn't track state transitions, however.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InteractionState {
    /// The component is hovered or pressed.
    Default,

    /// The user's cursor is hovering over the component.
    Hovered,

    /// The user is clicking on the component.
    Pressed,
}

/// The `EventVisualReaction` specifies in which way the handler visually
/// reacted to the event.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub enum EventVisualReaction {
    Ignored,
    ContentUpdated,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseMoveEvent {
    pub reaction: EventVisualReaction,

    pub position: Position<f32>,
    pub previous_position: Position<f32>,
    pub delta_x: f32,
    pub delta_y: f32,
}
