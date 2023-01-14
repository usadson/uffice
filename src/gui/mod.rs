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

    pub fn size(&self) -> Position<T> {
        Position::new(self.right - self.left, self.bottom - self.top)
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// An RGBA color.
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {

    pub const BLACK: Color = Color::from_rgb(0, 0, 0);
    pub const WHITE: Color = Color::from_rgb(0, 0, 0);

    pub const RED: Color = Color::from_rgb(255, 0, 0);
    pub const GREEN: Color = Color::from_rgb(0, 255, 0);
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
    /// A certain tab was loading and is now ready.
    TabBecameReady(TabId),
}

unsafe impl Send for AppEvent {}
