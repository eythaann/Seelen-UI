use std::collections::HashMap;

use seelen_core::{
    resource::WidgetId,
    state::{WidgetInstanceMode, WidgetLoader},
    system_state::MonitorId,
};

use crate::{
    error::Result,
    resources::RESOURCES,
    state::application::{FullState, FULL_STATE},
    widgets::{
        loader::WidgetInstance, toolbar::FancyToolbar, weg::SeelenWeg,
        window_manager::instance::WindowManagerV2,
    },
    windows_api::monitor::MonitorView,
};

/// This struct stores the widgets of a monitor
pub struct SluMonitorInstance {
    pub view: MonitorView,
    pub main_target_id: MonitorId,
    // legacy widgets
    pub toolbar: Option<FancyToolbar>,
    pub weg: Option<SeelenWeg>,
    pub wm: Option<WindowManagerV2>,
    // new widgets storage
    pub widgets: HashMap<WidgetId, WidgetInstance>,
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

    pub fn reload_widgets(&mut self) -> Result<()> {
        // unload uninstalled widgets
        self.widgets
            .retain(|key, _| RESOURCES.widgets.contains(key));

        let mut to_load = Vec::new();
        RESOURCES.widgets.scan(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                to_load.push((k.clone(), w.clone()));
            }
        });

        let state = FULL_STATE.load();
        for (key, widget) in to_load {
            if !state.is_widget_enable_on_monitor(&widget, &self.main_target_id) {
                self.widgets.remove(&key); // unload disabled widgets
                continue;
            }

            if !self.widgets.contains_key(&key) {
                let monitor_id: Option<&str> = match widget.instances {
                    WidgetInstanceMode::ReplicaByMonitor => Some(&self.main_target_id),
                    _ => None,
                };
                self.widgets
                    .insert(key.clone(), WidgetInstance::load(&widget, monitor_id)?);
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

        self.reload_widgets()?;
        Ok(())
    }
}

unsafe impl Send for SluMonitorInstance {}
unsafe impl Sync for SluMonitorInstance {}
