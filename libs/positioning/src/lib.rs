mod api;
pub mod easings;
pub mod error;
pub mod minimization;
pub mod rect;

use std::collections::HashMap;

use windows::Win32::UI::WindowsAndMessaging::WM_SETREDRAW;

use crate::{
    api::{force_redraw_window, get_window_rect, is_explorer, position_window, send_message},
    easings::Easing,
    error::Result,
    rect::Rect,
};

#[derive(Debug, Default)]
pub struct Positioner {
    /// key-pair of window id and its desired position
    pub to_positioning: HashMap<isize, Rect>,
}

pub struct WinDataForAnimation {
    hwnd: isize,
    from: Rect,
    to: Rect,
    is_size_changing: bool,
    is_explorer: bool,
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
        for (window_id, rect) in self.to_positioning.iter() {
            position_window(*window_id, rect, true, false)?;
        }
        Ok(())
    }

    /// Place all windows to their desired position with animation,
    /// this will lock the thread until the animation is finished.
    pub fn place_animated(
        &self,
        duration_ms: u64,
        easing: Easing,
        on_end: impl FnOnce(Result<bool>) + Send + 'static,
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
        F: FnOnce(Result<bool>) + Send + 'static,
    {
        self.interrupt(); // interrupt previous animation if any
        self.wait(); // ensure previous run of this animation is finished
        let mut list = Vec::new();

        for (win_id, desired_rect) in self.batch.iter() {
            let initial_rect = get_window_rect(*win_id)?;
            let is_size_changing = initial_rect.width != desired_rect.width
                || initial_rect.height != desired_rect.height;
            let is_position_changing =
                initial_rect.x != desired_rect.x || initial_rect.y != desired_rect.y;
            // skip windows that are already in the desired position
            if !is_size_changing && !is_position_changing {
                continue;
            }
            list.push(WinDataForAnimation {
                hwnd: *win_id,
                from: initial_rect,
                to: *desired_rect,
                is_size_changing,
                is_explorer: is_explorer(*win_id)?,
            });
        }

        // there is nothing to animate
        if list.is_empty() {
            return Ok(());
        }

        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let easing = self.easing;
        let animation_duration = std::time::Duration::from_millis(self.duration_ms);
        self.animation_interrupt_signal = Some(tx);
        self.animation_thread = Some(std::thread::spawn(move || {
            for data in &list {
                if data.is_explorer {
                    send_message(data.hwnd, WM_SETREDRAW, Some(0), None);
                }
            }

            let result = Self::perform(&list, easing, animation_duration, rx);
            if result.as_ref().is_ok_and(|interrupted| !interrupted) {
                for data in &list {
                    if data.is_explorer {
                        send_message(data.hwnd, WM_SETREDRAW, Some(1), None);
                        let _ = force_redraw_window(data.hwnd);
                    }
                }
            }

            on_end(result);
        }));
        Ok(())
    }

    /// returns true if animation was interrupted/canceled
    fn perform(
        list: &Vec<WinDataForAnimation>,
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

            let rects = list.iter().map(|data| {
                (
                    data.hwnd,
                    keyframe::ease(easing, data.from, data.to, progress),
                    data.is_size_changing,
                )
            });

            // let mut hdwp = start_defered_positioning(list.len() as i32)?;
            for (window_id, rect, size_changing) in rects {
                // move_window(window_id, &rect, false)?;
                position_window(window_id, &rect, false, !size_changing)?;
                // hdwp = defer_window_position(hdwp, window_id, &rect, !size_changing)?;
            }
            //  finish_defered_positioning(hdwp)?;

            frames += 1;
            let elapsed = last_frame_time.elapsed();
            if elapsed < min_frame_duration {
                std::thread::sleep(min_frame_duration - elapsed);
            }
            last_frame_time = std::time::Instant::now();
        }

        if !interrupted {
            log::trace!("Animation completed in {frames} frames");
            for data in list {
                // defer window position doesn't force the desired size, it is locked by window min size
                position_window(data.hwnd, &data.to, true, false)?;
                let _ = force_redraw_window(data.hwnd);
            }
        }

        Ok(interrupted)
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
