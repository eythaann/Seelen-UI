//! OS contract of the crate: everything the animation engine (`lib.rs`) needs from the native
//! window system. Window ids are the native handle as an opaque `isize`.

use std::time::Instant;

use crate::{error::Result, rect::Rect};

/// Native window operations used to place and animate windows of other processes.
///
/// Stateless: every item is an associated function, the native resources with a lifetime
/// (timers, overlays, cloaks) are returned as owned values that release themselves on `Drop`.
pub trait PositioningSystem {
    type FrameTimer: FrameTimer;
    type Overlay: ThumbnailOverlay;
    /// Keeps the window cloaked (invisible but still composed) while alive.
    type Cloak;

    /// Outer rect of the window, in physical screen coordinates.
    fn window_rect(window: isize) -> Result<Rect>;

    /// App specific behaviors the animation must adapt to.
    fn window_quirks(window: isize) -> Result<WindowQuirks>;

    /// Moves and/or resizes the window, never activating it nor changing its z-order.
    fn place_window(window: isize, rect: &Rect, options: PlaceOptions) -> Result<()>;

    /// Forces a full repaint of the window (frame and children included).
    fn redraw_window(window: isize) -> Result<()>;

    /// Raises the priority of the calling (animation) thread.
    fn boost_current_thread_priority();

    fn create_frame_timer() -> Result<Self::FrameTimer>;

    /// Creates an overlay showing a live thumbnail of `source`, initially at `initial`.
    /// Must be used and dropped on the thread that created it.
    fn create_overlay(source: isize, initial: &Rect) -> Result<Self::Overlay>;

    fn cloak(window: isize) -> Result<Self::Cloak>;
}

/// High precision timer used to pace the animation frames.
pub trait FrameTimer {
    /// Blocks until `target`. Returns immediately if it is already in the past.
    fn sleep_until(&self, target: Instant);
}

/// Composited copy of a window that can be stretched without the window itself being touched.
pub trait ThumbnailOverlay {
    /// Places the thumbnail where a window with outer rect `rect` would show its content.
    fn set_destination(&self, rect: &Rect) -> Result<()>;

    /// Processes the pending native messages of the overlay (the animation thread has no loop).
    fn pump_messages(&self);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlaceOptions {
    /// Repaint the window after placing it.
    pub redraw: bool,
    /// Only move it, keeping its current size.
    pub keep_size: bool,
    /// Queue the request to the window's thread instead of waiting for it to be processed.
    /// Queued requests are applied in order, but after any synchronous one issued meanwhile.
    pub asynchronous: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WindowQuirks {
    /// The window has to be redrawn on every frame of a realtime animation.
    pub redraw_each_frame: bool,
    /// The app stops rendering while cloaked, so it must not be resized until uncloaked.
    pub pauses_rendering_while_cloaked: bool,
    /// The app misbehaves when its windows are cloaked, so it is never animated buffered.
    pub breaks_when_cloaked: bool,
}
