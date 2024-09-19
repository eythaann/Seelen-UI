use color_eyre::eyre::eyre;
use getset::{Getters, MutGetters};

use crate::{
    error_handler::Result, log_error, seelen_bar::FancyToolbar, seelen_wall::SeelenWall,
    seelen_weg::SeelenWeg, seelen_wm_v2::instance::WindowManagerV2, state::application::FullState,
    windows_api::WindowsApi,
};

use windows::Win32::Graphics::Gdi::HMONITOR;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Monitor {
    handle: HMONITOR,
    name: String,
    toolbar: Option<FancyToolbar>,
    weg: Option<SeelenWeg>,
    wm: Option<WindowManagerV2>,
    wall: Option<SeelenWall>,
}

unsafe impl Send for Monitor {}

impl Monitor {
    pub fn new(hmonitor: HMONITOR, settings: &FullState) -> Result<Self> {
        if hmonitor.is_invalid() {
            return Err(eyre!("Invalid Monitor").into());
        }
        let mut monitor = Self {
            handle: hmonitor,
            name: WindowsApi::monitor_name(hmonitor)?,
            toolbar: None,
            weg: None,
            wm: None,
            wall: None,
        };
        monitor.load_settings(settings)?;
        Ok(monitor)
    }

    pub fn update_handle(&mut self, id: HMONITOR) {
        self.handle = id;
        log_error!(self.ensure_positions());
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
        if let Some(wall) = &mut self.wall {
            wall.set_position()?;
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
            self.weg = Some(SeelenWeg::new(&self.name)?)
        }
        Ok(())
    }

    fn add_wm(&mut self) -> Result<()> {
        if self.wm.is_none() {
            self.wm = Some(WindowManagerV2::new(&self.name)?)
        }
        Ok(())
    }

    fn add_wall(&mut self) -> Result<()> {
        if self.wall.is_none() {
            self.wall = Some(SeelenWall::new()?)
        }
        Ok(())
    }

    pub fn load_settings(&mut self, settings: &FullState) -> Result<()> {
        if settings.is_bar_enabled() {
            self.add_toolbar()?;
        } else {
            self.toolbar = None;
        }

        if settings.is_weg_enabled() {
            self.add_weg()?;
        } else {
            self.weg = None;
        }

        if settings.is_window_manager_enabled() {
            self.add_wm()?;
        } else {
            self.wm = None;
        }

        if settings.is_wall_enabled() && self.handle == WindowsApi::primary_monitor() {
            self.add_wall()?;
        } else {
            self.wall = None;
        }

        self.ensure_positions()?;
        Ok(())
    }

    pub fn is_focused(&self) -> bool {
        let hwnd = WindowsApi::get_foreground_window();
        self.handle == WindowsApi::monitor_from_window(hwnd)
    }
}
