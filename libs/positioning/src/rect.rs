use keyframe::CanTween;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CanTween for Rect {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        Self {
            x: f64::ease(from.x as f64, to.x as f64, time).ceil() as i32,
            y: f64::ease(from.y as f64, to.y as f64, time).ceil() as i32,
            width: f64::ease(from.width as f64, to.width as f64, time).ceil() as i32,
            height: f64::ease(from.height as f64, to.height as f64, time).ceil() as i32,
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
