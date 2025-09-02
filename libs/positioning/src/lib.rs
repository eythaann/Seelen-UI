mod api;
pub mod easings;
pub mod error;
pub mod minimization;
pub mod rect;

use std::collections::HashMap;

use keyframe::{AnimationSequence, keyframes};

use crate::{
    api::{
        defer_window_position, finish_defered_positioning, force_redraw_window, get_window_rect,
        start_defered_positioning,
    },
    easings::Easing,
    error::{Error, Result},
    rect::Rect,
};

#[derive(Debug, Default)]
pub struct Positioner {
    /// key-pair of window id and its desired position
    pub to_positioning: HashMap<isize, Rect>,
}

impl Positioner {
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
        let mut hdwp = start_defered_positioning(self.to_positioning.len() as i32)?;
        for (window_id, rect) in self.to_positioning.iter() {
            hdwp = defer_window_position(hdwp, *window_id, rect, true)
                .map_err(|_| Error::SetPositionFailed)?;
        }
        finish_defered_positioning(hdwp)?;
        Ok(())
    }

    /// Place all windows to their desired position with animation,
    /// this will lock the thread until the animation is finished.
    pub fn place_animated(
        &self,
        duration_ms: u64,
        easing: Easing,
        on_end: impl FnOnce(Result<()>) + Send + 'static,
    ) -> Result<AppWinAnimation> {
        let mut animation = AppWinAnimation::new(duration_ms, easing);
        animation.batch = self.to_positioning.clone();
        animation.start(on_end)?;
        Ok(animation)
    }
}

pub struct AppWinAnimation {
    batch: HashMap<isize, Rect>,
    easing: Easing,
    duration_ms: u64,
    animation_interrupt_signal: Option<std::sync::mpsc::Sender<()>>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
}

impl AppWinAnimation {
    fn new(duration_ms: u64, easing: Easing) -> Self {
        Self {
            batch: HashMap::new(),
            easing,
            duration_ms,
            animation_interrupt_signal: None,
            animation_thread: None,
        }
    }

    fn start<F>(&mut self, on_end: F) -> Result<()>
    where
        F: FnOnce(Result<()>) + Send + 'static,
    {
        self.interrupt(); // interrupt previous animation if any
        self.wait(); // ensure previous run of this animation is finished
        let mut sequences = Vec::new();

        for (w_addr, desired_rect) in self.batch.iter() {
            let initial_rect = get_window_rect(*w_addr)?;

            // skip windows that are already in the desired position
            if &initial_rect == desired_rect {
                continue;
            }

            let sequence = keyframes![
                (initial_rect, 0.0, self.easing),
                (desired_rect.clone(), 1.0)
            ];
            sequences.push((*w_addr, sequence));
        }

        // there is nothing to animate
        if sequences.is_empty() {
            return Ok(());
        }

        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let animation_duration = std::time::Duration::from_millis(self.duration_ms);
        self.animation_interrupt_signal = Some(tx);
        self.animation_thread = Some(std::thread::spawn(move || {
            let result = Self::perform(sequences, animation_duration, rx);
            on_end(result);
        }));
        Ok(())
    }

    fn perform(
        mut sequences: Vec<(isize, AnimationSequence<Rect>)>,
        animation_duration: std::time::Duration,
        interrupt_rx: std::sync::mpsc::Receiver<()>,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut progress = 0.0;

        while progress < 1.0 && interrupt_rx.try_recv().is_err() {
            let elapsed = start_time.elapsed();
            progress = (elapsed.as_secs_f64() / animation_duration.as_secs_f64()).min(1.0);

            let mut hdwp = start_defered_positioning(sequences.len() as i32)?;
            for (window_id, sequence) in sequences.iter_mut() {
                sequence.advance_to(progress);
                let rect = sequence.now();
                hdwp = defer_window_position(hdwp, *window_id, &rect, false)
                    .map_err(|_| Error::SetPositionFailed)?;
            }
            finish_defered_positioning(hdwp)?;
        }

        // re-draw windows to remove artifacts
        for (window_id, _) in sequences {
            let _ = force_redraw_window(window_id);
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.animation_thread.is_some()
    }

    /// Interrupt the animation
    pub fn interrupt(&mut self) {
        if let Some(animation_interrupt_signal) = self.animation_interrupt_signal.take() {
            let _ = animation_interrupt_signal.send(());
        }
    }

    /// Wait for the animation to finish, and return the result of the last performed animation
    pub fn wait(&mut self) {
        if let Some(animation_thread) = self.animation_thread.take() {
            animation_thread
                .join()
                .expect("Join animation thread failed");
        }
    }
}
