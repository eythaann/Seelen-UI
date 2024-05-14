use color_eyre::eyre::eyre;
use getset::{Getters, MutGetters};

use crate::{
    error_handler::Result,
    seelen_bar::FancyToolbar,
    seelen_weg::SeelenWeg,
    state::State,
};

use windows::Win32::Graphics::Gdi::HMONITOR;

#[derive(Getters, MutGetters)]
pub struct Monitor {
    #[allow(dead_code)]
    hmonitor: HMONITOR,
    #[getset(get = "pub", get_mut = "pub")]
    toolbar: Option<FancyToolbar>,
    #[getset(get = "pub", get_mut = "pub")]
    weg: Option<SeelenWeg>,
}

impl Monitor {
    pub fn new(hmonitor: HMONITOR, settings: &State) -> Result<Self> {
        if hmonitor.is_invalid() {
            return Err(eyre!("Invalid Monitor").into());
        }

        let mut monitor = Self {
            hmonitor,
            toolbar: None,
            weg: None,
        };

        if settings.is_bar_enabled() {
            match FancyToolbar::new(hmonitor.0) {
                Ok(bar) => monitor.toolbar = Some(bar),
                Err(e) => log::error!("Failed to create Toolbar: {}", e),
            }
        }

        if settings.is_weg_enabled() {
            match SeelenWeg::new(hmonitor.0) {
                Ok(weg) => monitor.weg = Some(weg),
                Err(e) => log::error!("Failed to create SeelenWeg: {}", e),
            }
        }

        Ok(monitor)
    }
}
