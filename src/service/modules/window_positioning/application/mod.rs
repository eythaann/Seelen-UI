#[cfg(windows)]
mod windows;

use std::{collections::HashMap, sync::LazyLock};

use positioning::{AnimationMode, AnimationOrchestrator, PositionerBuilder, easings::Easing};
use seelen_core::rect::Rect;

use crate::error::Result;

use super::domain::WindowPositioningSystem;

#[cfg(windows)]
type Platform = windows::WindowsWindowPositioning;

pub struct WindowPositioningManager {
    platform: Platform,
    animations: AnimationOrchestrator,
}

impl WindowPositioningManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<WindowPositioningManager> =
            LazyLock::new(WindowPositioningManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            animations: AnimationOrchestrator::new(),
        }
    }

    pub fn show_window(&self, window: isize, command: i32, asynchronous: bool) -> Result<()> {
        self.platform.show_window(window, command, asynchronous)
    }

    pub fn set_position(&self, window: isize, rect: &Rect, flags: u32) -> Result<()> {
        self.platform.set_position(window, rect, flags)
    }

    pub fn set_foreground(&self, window: isize) -> Result<()> {
        self.platform.set_foreground(window)
    }

    /// Places all the windows at once, animated or not.
    pub fn place_batch(
        &self,
        list: HashMap<isize, Rect>,
        animated: bool,
        animation_duration: u64,
        easing: &str,
    ) -> Result<()> {
        let mut builder = PositionerBuilder::new();
        for (window, rect) in list {
            builder.add(
                window,
                positioning::rect::Rect {
                    x: rect.left,
                    y: rect.top,
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                },
            );
        }

        if !animated {
            builder.place()?;
            return Ok(());
        }

        let easing = Easing::from_name(easing).unwrap_or(Easing::Linear);
        // The orchestrator will automatically interrupt only the windows in this batch
        // if they're already animating, without affecting other windows
        self.animations.animate_batch(
            builder.build(),
            animation_duration,
            easing,
            AnimationMode::Buffered,
            move |result| {
                if let Err(err) = result {
                    log::error!("Animated window placement failed: {err}");
                }
            },
        )?;
        Ok(())
    }
}
