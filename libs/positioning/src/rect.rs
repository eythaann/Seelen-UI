use keyframe::CanTween;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CanTween for Rect {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        #[inline(always)]
        fn ease_field(from: i32, to: i32, time: impl keyframe::num_traits::Float) -> i32 {
            if from == to {
                to
            } else {
                f64::ease(from as f64, to as f64, time).ceil() as i32
            }
        }

        Self {
            x: ease_field(from.x, to.x, time),
            y: ease_field(from.y, to.y, time),
            width: ease_field(from.width, to.width, time),
            height: ease_field(from.height, to.height, time),
        }
    }
}

impl From<windows::Win32::Foundation::RECT> for Rect {
    fn from(rect: windows::Win32::Foundation::RECT) -> Self {
        Self {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    }
}

impl From<Rect> for windows::Win32::Foundation::RECT {
    fn from(rect: Rect) -> Self {
        Self {
            left: rect.x,
            top: rect.y,
            right: rect.x + rect.width,
            bottom: rect.y + rect.height,
        }
    }
}
