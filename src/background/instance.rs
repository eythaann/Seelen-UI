use getset::{Getters, MutGetters};
use seelen_core::handlers::SeelenEvent;

use crate::{
    error_handler::Result,
    log_error,
    seelen_bar::FancyToolbar,
    seelen_weg::SeelenWeg,
    seelen_wm_v2::instance::WindowManagerV2,
    state::application::FullState,
    windows_api::{monitor::Monitor, WindowsApi},
};

use windows::Win32::Graphics::Gdi::HMONITOR;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SeelenInstanceContainer {
    handle: HMONITOR,
    monitor: Monitor,
    name: String,
    toolbar: Option<FancyToolbar>,
    weg: Option<SeelenWeg>,
    wm: Option<WindowManagerV2>,
}

unsafe impl Send for SeelenInstanceContainer {}

impl SeelenInstanceContainer {
    pub fn new(hmonitor: HMONITOR, settings: &FullState) -> Result<Self> {
        if hmonitor.is_invalid() {
            return Err("Invalid Monitor".into());
        }
        let mut instance = Self {
            handle: hmonitor,
            monitor: Monitor::from(hmonitor),
            name: WindowsApi::monitor_name(hmonitor)?,
            toolbar: None,
            weg: None,
            wm: None,
        };
        instance.load_settings(settings)?;
        instance.ensure_positions()?;
        Ok(instance)
    }

    pub fn update_handle(&mut self, id: HMONITOR) {
        if self.handle != id {
            self.handle = id;
            self.monitor = Monitor::from(id);
        } else {
            #[allow(clippy::clone_on_copy)]
            let before_update = self.monitor.clone();
            self.monitor.update().ok();

            if *self.monitor.display_orientation() != *before_update.display_orientation() {
                self.propagate_orientation();
            }

            if *self.monitor.tablet_mode() != *before_update.tablet_mode() {
                self.propagate_tablet_mode();
            }
        }
    }

    fn propagate_orientation(&mut self) {
        let orientation = self.monitor.display_orientation();
        if let Some(bar) = &self.toolbar {
            log_error!(
                bar.propagate_associated_event(SeelenEvent::ToolbarOrientationChanged, orientation)
            );
        }
        if let Some(weg) = &self.weg {
            log_error!(
                weg.propagate_associated_event(SeelenEvent::WegOrientationChanged, orientation)
            );
        }
    }
    fn propagate_tablet_mode(&mut self) {
        if let Some(bar) = &self.toolbar {
            log_error!(bar.propagate_associated_event(
                SeelenEvent::ToolbarTabletModeChanged,
                *self.monitor.tablet_mode()
            ));
        }
        if let Some(weg) = &self.weg {
            log_error!(weg.propagate_associated_event(
                SeelenEvent::WegTabletModeChanged,
                *self.monitor.tablet_mode()
            ));
        }
    }

    pub fn ensure_positions(&mut self) -> Result<()> {
        if let Some(bar) = &mut self.toolbar {
            bar.set_position(self.handle)?;
        }
        if let Some(weg) = &mut self.weg {
            weg.set_position(self.handle)?;
        }
        if let Some(wm) = &mut self.wm {
            wm.set_position(self.handle)?;
        }
        Ok(())
    }

    fn add_toolbar(&mut self) -> Result<()> {
        if self.toolbar.is_none() {
            self.toolbar = Some(FancyToolbar::new(&self.name)?);
        }
        Ok(())
    }

    fn add_weg(&mut self) -> Result<()> {
        if self.weg.is_none() {
            self.weg = Some(SeelenWeg::new(&self.name)?);
        }
        Ok(())
    }

    fn add_wm(&mut self) -> Result<()> {
        if self.wm.is_none() {
            self.wm = Some(WindowManagerV2::new(&self.name)?)
        }
        Ok(())
    }

    pub fn load_settings(&mut self, settings: &FullState) -> Result<()> {
        if settings.is_bar_enabled_on_monitor(self.monitor.index()?) {
            self.add_toolbar()?;
        } else {
            self.toolbar = None;
        }

        if settings.is_weg_enabled_on_monitor(self.monitor.index()?) {
            self.add_weg()?;
        } else {
            self.weg = None;
        }

        if settings.is_window_manager_enabled() {
            self.add_wm()?;
        } else {
            self.wm = None;
        }
        Ok(())
    }

    pub fn is_focused(&self) -> bool {
        let hwnd = WindowsApi::get_foreground_window();
        self.handle == WindowsApi::monitor_from_window(hwnd)
    }
}
