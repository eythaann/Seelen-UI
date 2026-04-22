pub mod hook;

use crate::{error::Result, state::application::FULL_STATE, windows_api::monitor::Monitor};

pub struct FancyToolbar {}

// statics
impl FancyToolbar {
    pub fn get_toolbar_height_on_monitor(monitor: &Monitor) -> Result<i32> {
        let state = FULL_STATE.load();
        let settings = &state.settings.by_widget.fancy_toolbar;
        let scale_factor = monitor.scale_factor()?;
        Ok((settings.total_size() as f64 * scale_factor) as i32)
    }
}
