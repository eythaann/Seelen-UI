use std::{sync::atomic::Ordering, time::Duration};

use seelen_core::{Point, handlers::SeelenEvent};

use crate::{
    app::emit_to_webviews,
    error::Result,
    utils::spawn_named_thread,
    windows_api::{event_window::IS_INTERACTIVE_SESSION, input::Mouse},
};

use super::CursorHookApi;

pub struct WindowsCursorHook;

impl CursorHookApi for WindowsCursorHook {
    fn position() -> Result<Point> {
        Mouse::get_cursor_pos()
    }

    fn start() {
        spawn_named_thread("MouseEventHook", || {
            let mut last_pos = Point::default();
            let sleep_time = Duration::from_millis(100); // 10fps
            loop {
                // Pause when session is not interactive to reduce CPU usage
                if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }

                if let Ok(pos) = Self::position()
                    && last_pos != pos
                {
                    emit_to_webviews(SeelenEvent::GlobalMouseMove, [pos.x, pos.y]);
                    last_pos = pos;
                }
                std::thread::sleep(sleep_time);
            }
        });
    }
}
