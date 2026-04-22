use crate::{error::Result, state::application::FULL_STATE, windows_api::monitor::Monitor};

pub struct SeelenWeg {}

impl SeelenWeg {
    pub fn get_weg_size_on_monitor(monitor: &Monitor) -> Result<i32> {
        let state = FULL_STATE.load();
        let settings: &seelen_core::state::SeelenWegSettings = &state.settings.by_widget.weg;
        let total_size = (settings.total_size() as f64 * monitor.scale_factor()?) as i32;
        Ok(total_size)
    }
}
