mod api;
pub mod easings;
pub mod error;
pub mod minimization;
mod overlay;
pub mod rect;
mod shell_view;
mod timer;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::{
    api::{
        force_redraw_window, get_class, get_window_rect, position_window, position_window_async,
    },
    easings::Easing,
    error::Result,
    overlay::ThumbnailOverlay,
    rect::Rect,
    shell_view::CloakGuard,
    timer::{FrameTimer, boost_current_thread_priority},
};

const FRAME_DURATION: std::time::Duration = std::time::Duration::from_millis(8); // ~120 fps cap

/// How a window animation is rendered.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    /// Moves/resizes the real window every frame. Content reflows live, but fps is bounded by
    /// how fast the target app handles each resize.
    #[default]
    Realtime,
    /// Resizes the real window once (cloaked) and animates a stretched DWM thumbnail of it.
    /// Frames are pure composition, so fps does not depend on the target app, but content is
    /// scaled during the animation instead of reflowing.
    Buffered,
}

#[derive(Debug, Default)]
pub struct PositionerBuilder {
    /// key-pair of window id and its desired position
    pub to_positioning: HashMap<isize, Rect>,
}

/// Whether two rects differ by more than `tolerance` pixels on any field.
/// Used to tell an OS-driven external move (e.g. WM_DPICHANGED resize) apart
/// from harmless rounding noise between the eased rect and the reported one.
fn rect_diff_exceeds(a: &Rect, b: &Rect, tolerance: i32) -> bool {
    (a.x - b.x).abs() > tolerance
        || (a.y - b.y).abs() > tolerance
        || (a.width - b.width).abs() > tolerance
        || (a.height - b.height).abs() > tolerance
}

struct WinDataForAnimation {
    hwnd: isize,
    from: Rect,
    to: Rect,
    is_size_changing: bool,
    is_explorer: bool,
    is_chromium: bool,
}

/// Message sent to a running animation thread.
enum AnimationSignal {
    /// Stop as soon as possible.
    Interrupt,
    /// Buffered only: continue from the rect currently on screen towards a new target, reusing
    /// the running thread, overlay, thumbnail and cloak instead of tearing them down.
    Retarget {
        to: Rect,
        easing: Easing,
        duration: std::time::Duration,
    },
}

/// One interpolation leg of a buffered animation; replaced on every retarget.
struct Segment {
    from: Rect,
    to: Rect,
    easing: Easing,
    start: std::time::Instant,
    secs: f64,
}

impl Segment {
    fn new(from: Rect, to: Rect, easing: Easing, duration: std::time::Duration) -> Self {
        Self {
            from,
            to,
            easing,
            start: std::time::Instant::now(),
            secs: duration.as_secs_f64(),
        }
    }

    fn progress(&self) -> f64 {
        if self.secs <= 0.0 {
            return 1.0;
        }
        (self.start.elapsed().as_secs_f64() / self.secs).min(1.0)
    }

    fn rect_at(&self, progress: f64) -> Rect {
        keyframe::ease(self.easing, self.from, self.to, progress)
    }
}

impl PositionerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, window_id: isize, rect: Rect) {
        self.to_positioning.insert(window_id, rect);
    }

    pub fn remove(&mut self, window_id: isize) {
        self.to_positioning.remove(&window_id);
    }

    pub fn clear(&mut self) {
        self.to_positioning.clear();
    }

    /// Place all windows to their desired position
    pub fn place(&self) -> Result<()> {
        for (window_id, rect) in self.to_positioning.iter() {
            position_window(*window_id, rect, true, false)?;
            let _ = force_redraw_window(*window_id);
        }
        Ok(())
    }

    /// Get the batch as a HashMap
    pub fn build(self) -> HashMap<isize, Rect> {
        self.to_positioning
    }
}

/// Manages the animation of a single window
pub struct WindowAnimation {
    hwnd: isize,
    signal: Option<std::sync::mpsc::Sender<AnimationSignal>>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
    /// True while the running (buffered) animation accepts [`AnimationSignal::Retarget`]. Only
    /// flipped under its lock, so a retarget is either seen by the thread or never sent.
    accepts_retarget: Arc<Mutex<bool>>,
}

impl WindowAnimation {
    fn new() -> Self {
        Self {
            hwnd: 0,
            signal: None,
            animation_thread: None,
            accepts_retarget: Arc::new(Mutex::new(false)),
        }
    }

    /// Start animating this window. A running buffered animation is retargeted in place;
    /// otherwise any running animation is interrupted and a new one started.
    fn start<F>(
        &mut self,
        hwnd: isize,
        target_rect: Rect,
        easing: Easing,
        duration_ms: u64,
        mode: AnimationMode,
        on_end: Arc<F>,
    ) -> Result<()>
    where
        F: Fn(Result<bool>) + Sync + Send + 'static,
    {
        let animation_duration = std::time::Duration::from_millis(duration_ms);
        if mode == AnimationMode::Buffered
            && self.try_retarget(target_rect, easing, animation_duration)
        {
            return Ok(());
        }

        // Interrupt any existing animation for this window
        self.interrupt();
        self.wait();

        self.hwnd = hwnd;

        // Get initial rect
        let initial_rect = get_window_rect(hwnd)?;
        let is_size_changing =
            initial_rect.width != target_rect.width || initial_rect.height != target_rect.height;
        let is_position_changing =
            initial_rect.x != target_rect.x || initial_rect.y != target_rect.y;

        // Skip if already in position
        if !is_size_changing && !is_position_changing {
            return Ok(());
        }

        let class = get_class(hwnd)?;
        let mut data = WinDataForAnimation {
            hwnd,
            from: initial_rect,
            to: target_rect,
            is_size_changing,
            is_explorer: class == "CabinetWClass" || class == "ExplorerWClass",
            is_chromium: class.starts_with("Chrome_WidgetWin_"),
        };

        let (tx, rx) = std::sync::mpsc::channel::<AnimationSignal>();
        let accepts_retarget = self.accepts_retarget.clone();
        *lock(&accepts_retarget) = mode == AnimationMode::Buffered;

        let thread = std::thread::spawn(move || {
            let result = match mode {
                AnimationMode::Realtime => Self::perform(&data, easing, animation_duration, &rx),
                AnimationMode::Buffered => Self::perform_buffered(
                    &data,
                    easing,
                    animation_duration,
                    &rx,
                    &accepts_retarget,
                )
                .or_else(|err| {
                    log::warn!("Buffered animation failed, falling back to realtime: {err}");
                    // Retargets sent before the gate closed must not be lost (the caller assumed
                    // they were applied), so the fallback heads to the latest one.
                    let mut easing = easing;
                    let mut duration = animation_duration;
                    *lock(&accepts_retarget) = false;
                    while let Ok(signal) = rx.try_recv() {
                        match signal {
                            AnimationSignal::Interrupt => return Ok(true),
                            AnimationSignal::Retarget {
                                to,
                                easing: e,
                                duration: d,
                            } => (data.to, easing, duration) = (to, e, d),
                        }
                    }
                    data.is_size_changing =
                        data.from.width != data.to.width || data.from.height != data.to.height;
                    Self::perform(&data, easing, duration, &rx)
                }),
            };
            on_end(result);
        });

        self.signal = Some(tx);
        self.animation_thread = Some(thread);

        Ok(())
    }

    /// Hands a new target to the running buffered animation. Returns false when there is none
    /// accepting it (finished, finishing, realtime or failed), so a new one must be started.
    fn try_retarget(&self, to: Rect, easing: Easing, duration: std::time::Duration) -> bool {
        let accepts = lock(&self.accepts_retarget);
        *accepts
            && self.signal.as_ref().is_some_and(|tx| {
                tx.send(AnimationSignal::Retarget {
                    to,
                    easing,
                    duration,
                })
                .is_ok()
            })
    }

    /// Returns true if animation was interrupted/canceled
    fn perform(
        data: &WinDataForAnimation,
        easing: Easing,
        animation_duration: std::time::Duration,
        interrupt_rx: &std::sync::mpsc::Receiver<AnimationSignal>,
    ) -> Result<bool> {
        boost_current_thread_priority();
        let timer = FrameTimer::new()?;

        let animation_secs = animation_duration.as_secs_f64();
        let start_time = std::time::Instant::now();
        let mut interrupted = false;
        let mut last_rect = data.from;
        let mut frame_i = 0u32;
        let mut frames = 0u32;
        let mut resize_this_frame = true;

        // The active interpolation segment. Rebased whenever the window's actual rect
        // drifts from what we last set (e.g. the OS resizing it synchronously on a
        // WM_DPICHANGED when crossing a monitor boundary), so we resume smoothly from
        // wherever the window actually is instead of snapping it back mid-flight.
        let mut segment_from = data.from;
        let mut segment_start = start_time;
        let mut segment_secs = animation_secs;

        loop {
            if interrupt_rx.try_recv().is_ok() {
                interrupted = true;
                break;
            }

            let elapsed = start_time.elapsed();
            let overall_progress = (elapsed.as_secs_f64() / animation_secs).min(1.0);

            if overall_progress < 1.0
                && let Ok(actual_rect) = get_window_rect(data.hwnd)
                && rect_diff_exceeds(&actual_rect, &last_rect, 10)
            {
                segment_from = actual_rect;
                segment_start = std::time::Instant::now();
                segment_secs = animation_duration
                    .saturating_sub(elapsed)
                    .max(std::time::Duration::from_millis(1))
                    .as_secs_f64();
                last_rect = actual_rect;
            }

            let segment_progress = if overall_progress >= 1.0 {
                1.0
            } else {
                (segment_start.elapsed().as_secs_f64() / segment_secs).min(1.0)
            };

            let is_last_frame = overall_progress >= 1.0;
            let eased = keyframe::ease(easing, segment_from, data.to, segment_progress);

            // Resizing forces the target app to relayout + repaint (WM_NCCALCSIZE/WM_SIZE), while
            // moving is just a DWM surface offset. Interleave: move every frame, resize every
            // other frame, always resizing on the last one so the window lands on its target.
            let apply_size = data.is_size_changing && (resize_this_frame || is_last_frame);
            resize_this_frame = !apply_size;
            let rect = if apply_size {
                eased
            } else {
                Rect {
                    width: last_rect.width,
                    height: last_rect.height,
                    ..eased
                }
            };

            // Skip SetWindowPos when the interpolated pixel position didn't change —
            // common at easing tails where speed < 1px per frame.
            if rect != last_rect {
                position_window(data.hwnd, &rect, data.is_explorer, !apply_size)?;
                last_rect = rect;
                frames += 1;
            }

            if is_last_frame {
                break;
            }

            // Check interrupt before sleeping to keep abort latency < 1 SetWindowPos call.
            if interrupt_rx.try_recv().is_ok() {
                interrupted = true;
                break;
            }

            // Absolute target prevents per-frame jitter from accumulating across the animation.
            frame_i += 1;
            timer.sleep_until(start_time + FRAME_DURATION * frame_i);
        }

        if !interrupted {
            log::trace!(
                "Animation({:?}) completed: {} ticks, {} unique pixel frames",
                data.hwnd,
                frame_i,
                frames
            );
            let _ = force_redraw_window(data.hwnd);
        }

        Ok(interrupted)
    }

    /// Buffered variant of [`Self::perform`]: the real window is cloaked and placed on its target
    /// rect with a posted (non-blocking) SetWindowPos, while a DWM thumbnail of it is stretched
    /// from the initial rect to the target on an overlay. Each frame is a single
    /// DwmUpdateThumbnailProperties call, so the target app is never awaited during the animation.
    /// New targets arriving meanwhile ([`AnimationSignal::Retarget`]) continue from the rect on
    /// screen, reusing the thread, overlay, thumbnail and cloak.
    ///
    /// Returns true if animation was interrupted/canceled
    fn perform_buffered(
        data: &WinDataForAnimation,
        easing: Easing,
        animation_duration: std::time::Duration,
        signals: &std::sync::mpsc::Receiver<AnimationSignal>,
        accepts_retarget: &Mutex<bool>,
    ) -> Result<bool> {
        boost_current_thread_priority();
        let timer = FrameTimer::new()?;

        // Declaration order matters: guards drop in reverse, so the real window is uncloaked
        // before the overlay flushes and removes the thumbnail.
        let overlay = ThumbnailOverlay::create(data.hwnd, &data.from)?;
        // No DwmFlush in between: the thumbnail reaches DWM before the cloak request does (the
        // calls are sequential), so it is never composed after the cloak. Saves up to a vsync.
        let cloak = CloakGuard::cloak(data.hwnd)?;

        // Chromium/Electron suspend rendering while their HWND reports any cloak state, so a
        // resize done now would leave a stale/black surface. For them the thumbnail stretches the
        // start-size content and the real resize happens at the end, right before uncloaking.
        // Everyone else is resized upfront, so the thumbnail shows target-size content that lands
        // 1:1 (crisp) on the last frame.
        let pre_resize = !data.is_chromium;
        // Last rect requested to the real window; posted requests may still be queued.
        let mut placed = data.from;
        let mut segment = Segment::new(data.from, data.to, easing, animation_duration);
        if pre_resize {
            placed = Self::place(data.hwnd, &placed, &segment.to, true)?;
        }

        let start_time = std::time::Instant::now();
        let mut interrupted = false;
        let mut shown = data.from;
        let mut frame_i = 0u32;
        let mut frames = 0u32;

        'frames: loop {
            while let Ok(signal) = signals.try_recv() {
                if !Self::apply_signal(
                    signal,
                    data.hwnd,
                    pre_resize,
                    shown,
                    &mut segment,
                    &mut placed,
                )? {
                    interrupted = true;
                    break 'frames;
                }
            }

            let progress = segment.progress();
            let rect = segment.rect_at(progress);
            if rect != shown {
                overlay.set_destination(&rect)?;
                shown = rect;
                frames += 1;
            }
            overlay.pump_messages();

            if progress >= 1.0 {
                // Close the gate, then drain once more: a retarget sent right before closing
                // would otherwise be lost.
                let mut accepts = lock(accepts_retarget);
                match signals.try_recv() {
                    Ok(signal) => {
                        drop(accepts);
                        if !Self::apply_signal(
                            signal,
                            data.hwnd,
                            pre_resize,
                            shown,
                            &mut segment,
                            &mut placed,
                        )? {
                            interrupted = true;
                            break;
                        }
                        continue;
                    }
                    Err(_) => {
                        *accepts = false;
                        break;
                    }
                }
            }

            frame_i += 1;
            timer.sleep_until(start_time + FRAME_DURATION * frame_i);
        }

        // From here placements are synchronous when possible: the window is about to be shown,
        // and the next animation reads its rect as starting point. A sent (sync) SetWindowPos is
        // processed before already posted ones, so it is only safe once those landed.
        let actual = get_window_rect(data.hwnd)?;
        if interrupted {
            *lock(accepts_retarget) = false;
            // Leave the real window where the user last saw it so the next animation starts from
            // there instead of jumping.
            if pre_resize && actual != placed {
                // Still queued: line up behind them instead of being overridden by them.
                Self::place(data.hwnd, &placed, &shown, true)?;
            } else {
                Self::place(data.hwnd, &actual, &shown, false)?;
            }
        } else if actual != segment.to {
            // Chromium's deferred placement, or a slow app that hasn't processed the posted one
            // yet. Anything still queued ends on this same rect, so it is harmless when it lands.
            Self::place(data.hwnd, &actual, &segment.to, false)?;
        }

        drop(cloak);
        drop(overlay);

        if !interrupted {
            log::trace!(
                "Buffered animation({:?}) completed: {} ticks, {} unique pixel frames",
                data.hwnd,
                frame_i,
                frames
            );
        }

        Ok(interrupted)
    }

    /// Applies a signal received by a buffered animation. Returns false on interrupt.
    fn apply_signal(
        signal: AnimationSignal,
        hwnd: isize,
        pre_resize: bool,
        shown: Rect,
        segment: &mut Segment,
        placed: &mut Rect,
    ) -> Result<bool> {
        match signal {
            AnimationSignal::Interrupt => Ok(false),
            AnimationSignal::Retarget {
                to,
                easing,
                duration,
            } => {
                *segment = Segment::new(shown, to, easing, duration);
                if pre_resize {
                    *placed = Self::place(hwnd, placed, &to, true)?;
                }
                Ok(true)
            }
        }
    }

    /// Places the window currently at `current` on `rect` with a single redrawn resize, returning
    /// `rect`. Moves first and sizes after: if the move lands on a monitor with another DPI, the
    /// app applies its WM_DPICHANGED suggested size during the move, and our resize then
    /// overrides it with the real target. With `r#async` both requests are posted (applied in
    /// order) instead of awaited.
    fn place(hwnd: isize, current: &Rect, rect: &Rect, r#async: bool) -> Result<Rect> {
        if current == rect {
            return Ok(*rect);
        }
        let place = if r#async {
            position_window_async
        } else {
            position_window
        };
        if current.x != rect.x || current.y != rect.y {
            place(hwnd, rect, false, true)?;
        }
        place(hwnd, rect, true, false)?;
        Ok(*rect)
    }

    pub fn is_running(&self) -> bool {
        self.animation_thread.is_some()
    }

    fn interrupt(&mut self) {
        if let Some(signal) = self.signal.take() {
            let _ = signal.send(AnimationSignal::Interrupt);
        }
    }

    fn wait(&mut self) {
        if let Some(thread) = self.animation_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for WindowAnimation {
    fn drop(&mut self) {
        self.interrupt();
        self.wait();
    }
}

/// Orchestrates animations for multiple windows, allowing per-window interruption
pub struct AnimationOrchestrator {
    animations: scc::HashMap<isize, WindowAnimation>,
}

impl AnimationOrchestrator {
    pub fn new() -> Self {
        Self {
            animations: scc::HashMap::new(),
        }
    }

    /// Animate a batch of windows with the given duration and easing.
    /// If a window in the batch is already animating, it will be interrupted and restarted.
    /// Other windows not in the batch will continue animating uninterrupted.
    pub fn animate_batch<F>(
        &self,
        batch: HashMap<isize, Rect>,
        duration_ms: u64,
        easing: Easing,
        mode: AnimationMode,
        on_end: F,
    ) -> Result<()>
    where
        F: Fn(Result<bool>) + Sync + Send + 'static,
    {
        let on_end = Arc::new(on_end);
        for (hwnd, rect) in batch {
            self.animate_window(hwnd, rect, duration_ms, easing, mode, on_end.clone())?;
        }
        Ok(())
    }

    fn animate_window<F>(
        &self,
        hwnd: isize,
        target_rect: Rect,
        duration_ms: u64,
        easing: Easing,
        mode: AnimationMode,
        on_end: Arc<F>,
    ) -> Result<()>
    where
        F: Fn(Result<bool>) + Sync + Send + 'static,
    {
        // Start animation (this will interrupt any existing animation for this window only)
        let mut animation = self
            .animations
            .entry_sync(hwnd)
            .or_insert_with(WindowAnimation::new);
        animation.start(hwnd, target_rect, easing, duration_ms, mode, on_end)?;
        Ok(())
    }
}

impl Default for AnimationOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
