use std::{collections::HashMap, path::PathBuf};

use seelen_core::system_state::MonitorId;

use crate::{
    error::Result, state::application::FullState, widgets::third_party::WidgetInstance,
    widgets::toolbar::FancyToolbar, widgets::weg::SeelenWeg,
    widgets::window_manager::instance::WindowManagerV2, windows_api::monitor::MonitorView,
};

/// This struct stores the widgets of a monitor
pub struct SluMonitorInstance {
    pub view: MonitorView,
    pub main_target_id: MonitorId,
    pub toolbar: Option<FancyToolbar>,
    pub weg: Option<SeelenWeg>,
    pub wm: Option<WindowManagerV2>,
    /// third party widgets
    pub widgets: HashMap<PathBuf, WidgetInstance>,
}

impl SluMonitorInstance {
    pub fn new(view: MonitorView, settings: &FullState) -> Result<Self> {
        let main_target_id = view.primary_target()?.stable_id2()?;
        let mut instance = Self {
            view,
            main_target_id,
            toolbar: None,
            weg: None,
            wm: None,
            widgets: HashMap::new(),
        };
        instance.load_settings(settings)?;
        instance.ensure_positions()?;
        Ok(instance)
    }

    pub fn ensure_positions(&mut self) -> Result<()> {
        let win32_monitor = self.view.as_win32_monitor()?;

        if let Some(bar) = &mut self.toolbar {
            bar.set_position(win32_monitor.handle())?;
        }
        if let Some(weg) = &mut self.weg {
            weg.set_position(win32_monitor.handle())?;
        }
        if let Some(wm) = &mut self.wm {
            wm.set_position(win32_monitor.handle())?;
            WindowManagerV2::force_retiling()?;
        }
        Ok(())
    }

    fn add_toolbar(&mut self) -> Result<()> {
        if self.toolbar.is_none() {
            self.toolbar = Some(FancyToolbar::new(&self.main_target_id)?);
        }
        Ok(())
    }

    fn add_weg(&mut self) -> Result<()> {
        if self.weg.is_none() {
            self.weg = Some(SeelenWeg::new(&self.main_target_id)?);
        }
        Ok(())
    }

    fn add_wm(&mut self) -> Result<()> {
        if self.wm.is_none() {
            self.wm = Some(WindowManagerV2::new(&self.main_target_id)?)
        }
        Ok(())
    }

    pub fn reload_widgets(&mut self, state: &FullState) -> Result<()> {
        // unload uninstalled widgets
        self.widgets.retain(|id, _| state.widgets.contains_key(id));

        let third_party_widgets = state
            .widgets
            .iter()
            .filter(|(_, w)| !w.metadata.internal.bundled);
        for (id, widget) in third_party_widgets {
            if !state.is_widget_enable_on_monitor(widget, &self.main_target_id) {
                self.widgets.remove(id); // unload disabled widgets
                continue;
            }

            if !self.widgets.contains_key(id) {
                self.widgets.insert(
                    id.clone(),
                    WidgetInstance::load(widget.clone(), &self.main_target_id)?,
                );
            }
        }
        Ok(())
    }

    pub fn load_settings(&mut self, state: &FullState) -> Result<()> {
        if state.is_bar_enabled_on_monitor(&self.main_target_id) {
            self.add_toolbar()?;
        } else {
            self.toolbar = None;
        }

        if state.is_weg_enabled_on_monitor(&self.main_target_id) {
            self.add_weg()?;
        } else {
            self.weg = None;
        }

        if state.is_window_manager_enabled_on_monitor(&self.main_target_id) {
            self.add_wm()?;
        } else {
            self.wm = None;
        }

        self.reload_widgets(state)?;
        Ok(())
    }
}

unsafe impl Send for SluMonitorInstance {}
unsafe impl Sync for SluMonitorInstance {}
