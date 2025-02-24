use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::{
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_SETTINGCHANGE},
};

use crate::{
    error_handler::Result,
    event_manager, trace_lock,
    windows_api::{event_window::subscribe_to_background_window, MonitorEnumerator},
};

lazy_static! {
    pub static ref MONITOR_MANAGER: Arc<Mutex<MonitorManager>> = Arc::new(Mutex::new(
        MonitorManager::new().expect("Failed to create monitor manager")
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorManagerEvent {
    Added(String, HMONITOR),
    Removed(String, HMONITOR),
    Updated(String, HMONITOR),
}

pub struct MonitorManager {
    pub monitors: Vec<(String, HMONITOR)>,
}

unsafe impl Send for MonitorManager {}
unsafe impl Send for MonitorManagerEvent {}

event_manager!(MonitorManager, MonitorManagerEvent);

impl MonitorManager {
    fn window_proc(message: u32, _wparam: usize, _lparam: isize) -> Result<()> {
        match message {
            // Added based on this https://stackoverflow.com/a/33762334
            WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
                // log::debug!("Dispatching {}, {:?}, {:?}", message, wparam, lparam);
                let mut old_list = { trace_lock!(MONITOR_MANAGER).monitors.clone() };
                let new_list = Self::get_monitors()?;

                let sender = Self::event_tx();
                for (id, handle) in &new_list {
                    match old_list.iter().position(|x| x.0 == *id) {
                        Some(idx) => {
                            old_list.remove(idx);
                            sender.send(MonitorManagerEvent::Updated(id.clone(), *handle))?;
                        }
                        None => {
                            sender.send(MonitorManagerEvent::Added(id.clone(), *handle))?;
                        }
                    }
                }

                for (id, handle) in old_list {
                    sender.send(MonitorManagerEvent::Removed(id, handle))?;
                }

                trace_lock!(MONITOR_MANAGER).monitors = new_list.into_iter().collect();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn new() -> Result<Self> {
        subscribe_to_background_window(Self::window_proc);
        Ok(Self {
            monitors: Self::get_monitors()?,
        })
    }

    fn get_monitors() -> Result<Vec<(String, HMONITOR)>> {
        let mut monitors = Vec::new();
        for m in MonitorEnumerator::get_all_v2()? {
            if let Ok(id) = m.device_id() {
                monitors.push((id, m.handle()));
            }
        }
        Ok(monitors)
    }
}
