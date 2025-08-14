use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use parking_lot::Mutex;
use seelen_core::system_state::MonitorId;
use windows::{
    Devices::Display::Core::{
        DisplayManager, DisplayManagerChangedEventArgs, DisplayManagerOptions, DisplayState,
    },
    Foundation::TypedEventHandler,
    Win32::UI::WindowsAndMessaging::{WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_SETTINGCHANGE},
};

use crate::{
    error::Result,
    event_manager, log_error, trace_lock,
    windows_api::{event_window::subscribe_to_background_window, monitor::MonitorView},
};

pub static MONITOR_MANAGER: LazyLock<Arc<Mutex<MonitorManager>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(
        MonitorManager::create().expect("Failed to create monitor manager"),
    ))
});

pub static GLOBAL_DISPLAY_MANAGER: LazyLock<DisplayManager> =
    LazyLock::new(|| DisplayManager::Create(DisplayManagerOptions::None).unwrap());

pub struct MonitorManager {
    state: DisplayState,
    state_views: HashMap<MonitorId, MonitorView>,
    event_handler: TypedEventHandler<DisplayManager, DisplayManagerChangedEventArgs>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorManagerEvent {
    ViewAdded(MonitorView),
    ViewRemoved(MonitorId),
}

impl MonitorManager {
    fn create() -> Result<MonitorManager> {
        let event_handler = TypedEventHandler::new(Self::on_change);

        let state = GLOBAL_DISPLAY_MANAGER
            .TryReadCurrentStateForAllTargets()?
            .State()?;

        let mut state_views = HashMap::new();
        for view in state.Views()? {
            let view = MonitorView::from(view);
            state_views.insert(view.primary_target()?.stable_id2()?, view);
        }

        Ok(MonitorManager {
            state,
            state_views,
            event_handler,
        })
    }

    //  based on  https://stackoverflow.com/a/33762334
    fn window_proc(message: u32, wparam: usize, lparam: isize) -> Result<()> {
        match message {
            WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
                log::trace!("Monitors changed | {message} - {wparam} - {lparam}");
                Self::on_change(
                    &Some(DisplayManager::Create(DisplayManagerOptions::None)?),
                    &None,
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn init(&self) -> Result<()> {
        GLOBAL_DISPLAY_MANAGER.Changed(&self.event_handler)?;
        // this is failling so as workaround we are using window proc
        // GLOBAL_DISPLAY_MANAGER.Start()?;
        subscribe_to_background_window(Self::window_proc);
        Ok(())
    }

    fn on_change(
        sender: &Option<DisplayManager>,
        _args: &Option<DisplayManagerChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(sender) = sender {
            let mut guard = trace_lock!(MONITOR_MANAGER);

            let current_state = sender.TryReadCurrentStateForAllTargets()?.State()?;
            let mut current_views = HashMap::new();
            for view in current_state.Views()? {
                let view = MonitorView::from(view);
                let id = match view.primary_target().and_then(|t| t.stable_id2()) {
                    Ok(id) => id,
                    Err(_) => continue,
                };
                current_views.insert(id, view);
            }

            let mut old_views = std::mem::take(&mut guard.state_views);

            // new monitors were added
            for id in current_views.keys() {
                let was_already_present = old_views.remove(id).is_none();
                if was_already_present {
                    log_error!(Self::event_tx()
                        .send(MonitorManagerEvent::ViewAdded(current_views[id].clone())));
                }
            }

            // reciduals was removed/disconnected
            for (id, _) in old_views {
                log_error!(Self::event_tx().send(MonitorManagerEvent::ViewRemoved(id)));
            }

            guard.state = current_state;
            guard.state_views = current_views;
        }
        Ok(())
    }

    pub fn get_all_views() -> Result<Vec<MonitorView>> {
        let state = GLOBAL_DISPLAY_MANAGER.TryReadCurrentStateForAllTargets()?;
        let state = state.State()?;
        Ok(state.Views()?.into_iter().map(MonitorView::from).collect())
    }

    pub fn view_at(index: u32) -> Result<MonitorView> {
        let state = GLOBAL_DISPLAY_MANAGER.TryReadCurrentStateForAllTargets()?;
        let state = state.State()?;
        let view = state.Views()?.GetAt(index)?;
        Ok(view.into())
    }
}

event_manager!(MonitorManager, MonitorManagerEvent);
unsafe impl Send for MonitorManager {}
