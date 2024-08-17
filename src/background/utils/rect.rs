use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::RECT;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl From<RECT> for Rect {
    fn from(rect: RECT) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

impl From<Rect> for RECT {
    fn from(val: Rect) -> Self {
        RECT {
            left: val.left,
            top: val.top,
            right: val.right,
            bottom: val.bottom,
        }
    }
}

impl Eq for Rect {}
impl PartialEq for Rect {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left
            && self.top == other.top
            && self.right == other.right
            && self.bottom == other.bottom
    }
}
