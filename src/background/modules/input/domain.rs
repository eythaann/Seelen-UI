use std::fmt::Debug;
use std::fmt::Display;

use windows::Win32::Foundation::POINT;

use seelen_core::rect::Rect;

/// A Point type stores the x and y position.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Point(POINT);

impl Point {
    /// Creates a new position.
    pub fn new(x: i32, y: i32) -> Self {
        Self(POINT { x, y })
    }

    /// Retrieves the x position.
    pub fn get_x(&self) -> i32 {
        self.0.x
    }

    /// Retrieves the y position.
    pub fn get_y(&self) -> i32 {
        self.0.y
    }

    pub fn is_inside_rect(&self, rect: &Rect) -> bool {
        self.0.x >= rect.left
            && self.0.x <= rect.right
            && self.0.y >= rect.top
            && self.0.y <= rect.bottom
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.0.x)
            .field("y", &self.0.y)
            .finish()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0.x, self.0.y)
    }
}

impl From<POINT> for Point {
    fn from(point: POINT) -> Self {
        Self(point)
    }
}

impl From<Point> for POINT {
    fn from(val: Point) -> Self {
        val.0
    }
}

impl AsRef<POINT> for Point {
    fn as_ref(&self) -> &POINT {
        &self.0
    }
}

impl AsMut<POINT> for Point {
    fn as_mut(&mut self) -> &mut POINT {
        &mut self.0
    }
}
