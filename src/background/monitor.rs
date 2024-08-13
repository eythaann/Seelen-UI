use color_eyre::eyre::eyre;
use getset::{Getters, MutGetters};

use crate::{
    error_handler::Result, log_error, seelen_bar::FancyToolbar, seelen_weg::SeelenWeg,
    seelen_wm::WindowManager, state::State, utils::sleep_millis, windows_api::WindowsApi,
};

use windows::Win32::Graphics::Gdi::HMONITOR;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Monitor {
    handle: HMONITOR,
    name: String,
    toolbar: Option<FancyToolbar>,
    weg: Option<SeelenWeg>,
    wm: Option<WindowManager>,
}

impl Monitor {
    pub fn update_handle(&mut self, id: HMONITOR) {
        self.handle = id;
        log_error!(self.ensure_positions());
    }

    pub fn ensure_positions(&mut self) -> Result<()> {
        if let Some(bar) = &mut self.toolbar {
            bar.set_positions(self.handle.0)?;
        }
        Ok(())
    }

    pub fn new(hmonitor: HMONITOR, settings: &State) -> Result<Self> {
        if hmonitor.is_invalid() {
            return Err(eyre!("Invalid Monitor").into());
        }

        let mut monitor = Self {
            handle: hmonitor,
            name: WindowsApi::monitor_name(hmonitor)?,
            toolbar: None,
            weg: None,
            wm: None,
        };

        if settings.is_bar_enabled() {
            // Tauri can fail the on creation of the first window, thats's why we only should retry
            // for the first window created, the next windows should work normally.
            // Update(08/13/2024): I think this can be removed on recent tauri versions
            for attempt in 1..4 {
                match FancyToolbar::new(&monitor.name) {
                    Ok(bar) => {
                        monitor.toolbar = Some(bar);
                        break;
                    }
                    Err(e) => {
                        log::error!("Failed to create Toolbar (attempt {}): {}", attempt, e);
                        sleep_millis(30);
                    }
                }
            }
        }

        if settings.is_weg_enabled() {
            match SeelenWeg::new(hmonitor.0) {
                Ok(weg) => monitor.weg = Some(weg),
                Err(e) => log::error!("Failed to create SeelenWeg: {}", e),
            }
        }

        if settings.is_window_manager_enabled() && hmonitor == WindowsApi::primary_monitor() {
            match WindowManager::new(hmonitor.0) {
                Ok(wm) => monitor.wm = Some(wm),
                Err(e) => log::error!("Failed to create WindowManager: {}", e),
            }
        }

        monitor.ensure_positions()?;
        Ok(monitor)
    }

    pub fn is_focused(&self) -> bool {
        let hwnd = WindowsApi::get_foreground_window();
        self.handle == WindowsApi::monitor_from_window(hwnd)
    }

    pub fn is_ready(&self) -> bool {
        if let Some(weg) = &self.weg {
            if !weg.ready() {
                return false;
            }
        }

        if let Some(wm) = &self.wm {
            if !wm.ready() {
                return false;
            }
        }

        true
    }
}
