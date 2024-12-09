use std::collections::HashMap;

use getset::{Getters, MutGetters};
use seelen_core::state::WidgetId;

use crate::{
    error_handler::Result,
    log_error,
    seelen_bar::FancyToolbar,
    seelen_weg::SeelenWeg,
    seelen_wm_v2::instance::WindowManagerV2,
    state::application::FullState,
    widget_loader::WidgetInstance,
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
    /// third party widgets
    widgets: HashMap<WidgetId, WidgetInstance>,
}

unsafe impl Send for SeelenInstanceContainer {}

impl SeelenInstanceContainer {
    pub fn new(hmonitor: HMONITOR, settings: &FullState) -> Result<Self> {
        if hmonitor.is_invalid() {
            return Err("Invalid Monitor".into());
        }
        let monitor = Monitor::from(hmonitor);
        let mut instance = Self {
            handle: hmonitor,
            name: monitor.display_device()?.id,
            monitor,
            toolbar: None,
            weg: None,
            wm: None,
            widgets: HashMap::new(),
        };
        instance.load_settings(settings)?;
        instance.ensure_positions()?;
        Ok(instance)
    }

    pub fn update_handle(&mut self, id: HMONITOR) {
        self.handle = id;
        self.monitor = Monitor::from(id);
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
        if settings.is_bar_enabled_on_monitor(self.monitor()) {
            self.add_toolbar()?;
        } else {
            self.toolbar = None;
        }

        if settings.is_weg_enabled_on_monitor(self.monitor()) {
            self.add_weg()?;
        } else {
            self.weg = None;
        }

        if settings.is_window_manager_enabled() {
            self.add_wm()?;
        } else {
            self.wm = None;
        }

        for (id, widget) in &settings.widgets {
            // Todo: filter by widget settings (enabled state)
            if self.widgets.contains_key(id) || widget.html.is_none() {
                continue;
            }
            self.widgets
                .insert(id.clone(), WidgetInstance::load(widget.clone())?);
        }

        Ok(())
    }

    pub fn is_focused(&self) -> bool {
        let hwnd = WindowsApi::get_foreground_window();
        self.handle == WindowsApi::monitor_from_window(hwnd)
    }
}
