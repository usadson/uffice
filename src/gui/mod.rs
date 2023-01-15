// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use crate::application::TabId;

pub mod animate;
pub mod app;
pub mod view;
pub mod painter;
pub mod scroll;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect<T> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T> Rect<T> where T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> {
    pub fn from_positions(left: T, right: T, top: T, bottom: T) -> Self {
        Self { left, right, top, bottom }
    }

    pub fn from_position_and_size(position: Position<T>, size: Size<T>) -> Self {
        Self {
            left: position.x(),
            right: position.x() + size.width(),
            top: position.y(),
            bottom: position.y() + size.height()
        }
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

    pub fn size(&self) -> Size<T> {
        Size::new(self.width(), self.height())
    }

    /// Get the width of the Rect.
    pub fn width(&self) -> T {
        self.right - self.left
    }

    /// Get the height of the Rect.
    pub fn height(&self) -> T {
        self.bottom - self.top
    }
}

impl Rect<f32> {
    /// Creates a Rect with no size.
    pub fn empty() -> Rect<f32> {
        Self::from_position_and_size(Position::new(0.0, 0.0), Size::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test() {
        assert_eq!(Rect::empty(), Rect::from_position_and_size(Position::new(0.0, 0.0), Size::new(0.0, 0.0)));

        assert_eq!(Rect::empty().bottom, 0.0);
        assert_eq!(Rect::empty().top, 0.0);
        assert_eq!(Rect::empty().right, 0.0);
        assert_eq!(Rect::empty().left, 0.0);

        assert_eq!(Rect::empty().width(), 0.0);
        assert_eq!(Rect::empty().height(), 0.0);

        assert_eq!(Rect::empty().position(), Position::new(0.0, 0.0));
        assert_eq!(Rect::empty().size(), Size::empty());
    }
}

/// Defines a size. Prefer this over using Vector2f for everything since it
/// communicates the definition better.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size<T> {
    width: T,
    height: T,
}

impl<T> Size<T> where T: Copy {
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

impl Size<f32> {
    /// Creates a size with no width or height.
    pub fn empty() -> Size<f32> {
        Self { width: 0.0, height: 0.0 }
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

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
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
    }
}

unsafe impl Send for AppEvent {}
