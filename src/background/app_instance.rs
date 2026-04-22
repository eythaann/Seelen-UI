use seelen_core::system_state::MonitorId;

use crate::{
    error::Result,
    modules::monitors::MonitorManager,
    state::application::FullState,
    widgets::{manager::WIDGET_MANAGER, weg::SeelenWeg, window_manager::instance::WindowManagerV2},
};

/// This struct stores the widgets for a display view
pub struct LegacyWidgetMonitorContainer {
    // the primary target id of the display view for this container was created
    pub view_primary_target_id: MonitorId,
    // legacy widgets
    pub weg: Option<SeelenWeg>,
    pub wm: Option<WindowManagerV2>,
}

impl LegacyWidgetMonitorContainer {
    pub fn new(view_primary_target_id: MonitorId, settings: &FullState) -> Result<Self> {
        let mut instance = Self {
            view_primary_target_id,
            weg: None,
            wm: None,
        };
        instance.load_settings(settings)?;
        instance.ensure_positions()?;
        Ok(instance)
    }

    pub fn ensure_positions(&mut self) -> Result<()> {
        let monitor = MonitorManager::instance()
            .get_display_view_for_target(&self.view_primary_target_id)?
            .as_win32_view()?;

        if let Some(weg) = &mut self.weg {
            weg.set_position(&monitor)?;
        }
        if let Some(wm) = &mut self.wm {
            wm.set_position(&monitor)?;
            WindowManagerV2::force_retiling()?;
        }
        Ok(())
    }

    fn add_weg(&mut self) -> Result<()> {
        if self.weg.is_none() {
            self.weg = Some(SeelenWeg::new(&self.view_primary_target_id)?);
        }
        Ok(())
    }

    fn add_wm(&mut self) -> Result<()> {
        if self.wm.is_none() {
            self.wm = Some(WindowManagerV2::new(&self.view_primary_target_id)?)
        }
        Ok(())
    }

    pub fn load_settings(&mut self, state: &FullState) -> Result<()> {
        if state.is_weg_enabled_on_monitor(&self.view_primary_target_id) {
            self.add_weg()?;
        } else {
            self.weg = None;
        }

        if state.is_window_manager_enabled_on_monitor(&self.view_primary_target_id) {
            self.add_wm()?;
        } else {
            self.wm = None;
        }

        WIDGET_MANAGER.reconcile()?;
        Ok(())
    }
}

unsafe impl Send for LegacyWidgetMonitorContainer {}
unsafe impl Sync for LegacyWidgetMonitorContainer {}
