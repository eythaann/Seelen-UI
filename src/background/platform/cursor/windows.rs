use std::time::Duration;

use seelen_core::{Point, handlers::SeelenEvent};

use crate::{
    app::emit_to_webviews, error::Result, utils::spawn_named_thread, windows_api::input::Mouse,
};

pub fn position() -> Result<Point> {
    Mouse::get_cursor_pos()
}

pub fn start_tracking() {
    spawn_named_thread("MouseEventHook", || {
        let mut last_pos = Point::default();
        let sleep_time = Duration::from_millis(100); // 10fps
        loop {
            // Pause when session is not interactive to reduce CPU usage
            if !crate::platform::session::is_interactive() {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }

            if let Ok(pos) = position()
                && last_pos != pos
            {
                emit_to_webviews(SeelenEvent::GlobalMouseMove, [pos.x, pos.y]);
                last_pos = pos;
            }
            std::thread::sleep(sleep_time);
        }
    });
}
