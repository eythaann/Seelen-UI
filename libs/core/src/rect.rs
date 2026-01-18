use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    /// The right edge (exclusive). Pixels at x >= right are outside the rectangle.
    pub right: i32,
    /// The bottom edge (exclusive). Pixels at y >= bottom are outside the rectangle.
    pub bottom: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Frame {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn as_frame(&self) -> Frame {
        Frame {
            x: self.left,
            y: self.top,
            width: (self.right - self.left) as u32,
            height: (self.bottom - self.top) as u32,
        }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn center(&self) -> Point {
        Point::new(self.left + self.width() / 2, self.top + self.height() / 2)
    }

    pub fn corners(&self) -> [Point; 4] {
        [
            Point::new(self.left, self.top),
            Point::new(self.right, self.top),
            Point::new(self.right, self.bottom),
            Point::new(self.left, self.bottom),
        ]
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        let right = self.right.min(other.right);
        let bottom = self.bottom.min(other.bottom);

        if left >= right || top >= bottom {
            return None;
        }

        Some(Rect {
            left,
            top,
            right,
            bottom,
        })
    }

    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.left && point.x < self.right && point.y >= self.top && point.y < self.bottom
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_squared(&self, other: &Point) -> i32 {
        let dx = self.x.saturating_sub(other.x);
        let dy = self.y.saturating_sub(other.y);
        dx.saturating_pow(2).saturating_add(dy.saturating_pow(2))
    }

    pub fn distance(&self, other: &Point) -> f64 {
        (self.distance_squared(other) as f64).sqrt()
    }
}
