mod api;
pub mod easings;
pub mod error;
pub mod minimization;
pub mod rect;

use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    api::{force_redraw_window, get_window_rect, is_explorer, position_window},
    easings::Easing,
    error::Result,
    rect::Rect,
};

#[derive(Debug, Default)]
pub struct PositionerBuilder {
    /// key-pair of window id and its desired position
    pub to_positioning: HashMap<isize, Rect>,
}

struct WinDataForAnimation {
    hwnd: isize,
    from: Rect,
    to: Rect,
    is_size_changing: bool,
    is_explorer: bool,
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
    interrupt_signal: Option<std::sync::mpsc::Sender<()>>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
}

impl WindowAnimation {
    fn new() -> Self {
        Self {
            hwnd: 0,
            interrupt_signal: None,
            animation_thread: None,
        }
    }

    /// Start animating this window. If already animating, interrupt and restart.
    fn start<F>(
        &mut self,
        hwnd: isize,
        target_rect: Rect,
        easing: Easing,
        duration_ms: u64,
        on_end: Arc<F>,
    ) -> Result<()>
    where
        F: Fn(Result<bool>) + Sync + Send + 'static,
    {
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

        let data = WinDataForAnimation {
            hwnd,
            from: initial_rect,
            to: target_rect,
            is_size_changing,
            is_explorer: is_explorer(hwnd)?,
        };

        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let animation_duration = std::time::Duration::from_millis(duration_ms);

        let thread = std::thread::spawn(move || {
            let result = Self::perform(&data, easing, animation_duration, rx);
            on_end(result);
        });

        self.interrupt_signal = Some(tx);
        self.animation_thread = Some(thread);

        Ok(())
    }

    /// Returns true if animation was interrupted/canceled
    fn perform(
        data: &WinDataForAnimation,
        easing: Easing,
        animation_duration: std::time::Duration,
        interrupt_rx: std::sync::mpsc::Receiver<()>,
    ) -> Result<bool> {
        let start_time = std::time::Instant::now();
        let mut progress = 0.0;
        let mut interrupted = false;

        let mut frames = 0;
        let mut last_frame_time = start_time;
        let min_frame_duration = std::time::Duration::from_millis(7); // ~ 144 fps as limit

        while progress < 1.0 {
            if interrupt_rx.try_recv().is_ok() {
                interrupted = true;
                break;
            }

            let elapsed = start_time.elapsed();
            progress =
                (elapsed.as_millis() as f64 / animation_duration.as_millis() as f64).min(1.0);

            let rect = keyframe::ease(easing, data.from, data.to, progress);
            position_window(data.hwnd, &rect, data.is_explorer, !data.is_size_changing)?;

            frames += 1;
            let elapsed = last_frame_time.elapsed();
            if elapsed < min_frame_duration {
                std::thread::sleep(min_frame_duration - elapsed);
            }
            last_frame_time = std::time::Instant::now();
        }

        if !interrupted {
            log::trace!("Animation({:?}) completed in {frames} frames", data.hwnd);
            let _ = force_redraw_window(data.hwnd);
        }

        Ok(interrupted)
    }

    pub fn is_running(&self) -> bool {
        self.animation_thread.is_some()
    }

    fn interrupt(&mut self) {
        if let Some(signal) = self.interrupt_signal.take() {
            let _ = signal.send(());
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
        on_end: F,
    ) -> Result<()>
    where
        F: Fn(Result<bool>) + Sync + Send + 'static,
    {
        let on_end = Arc::new(on_end);
        for (hwnd, rect) in batch {
            self.animate_window(hwnd, rect, duration_ms, easing, on_end.clone())?;
        }
        Ok(())
    }

    fn animate_window<F>(
        &self,
        hwnd: isize,
        target_rect: Rect,
        duration_ms: u64,
        easing: Easing,
        on_end: Arc<F>,
    ) -> Result<()>
    where
        F: Fn(Result<bool>) + Sync + Send + 'static,
    {
        // Start animation (this will interrupt any existing animation for this window only)
        let mut animation = self
            .animations
            .entry(hwnd)
            .or_insert_with(WindowAnimation::new);
        animation.start(hwnd, target_rect, easing, duration_ms, on_end)?;
        Ok(())
    }
}

impl Default for AnimationOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
