use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use getset::Getters;
use seelen_core::system_state::{ApplicationHistoryEntry, FocusedApp};

use lazy_static::lazy_static;

use crate::{
    error_handler::AppError,
    event_manager, log_error,
    windows_api::{window::Window, MonitorEnumerator},
};

lazy_static! {
    pub static ref APPLICATION_HISTORY: Arc<Mutex<ApplicationHistory>> = Arc::new(Mutex::new(
        ApplicationHistory::new().expect("Failed to create application history")
    ));
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ApplicationHistoryEvent {
    ApplicationHistoryAdded(FocusedApp),
    CurrentItemModified(FocusedApp),
    ApplicationHistoryChanged,
    ApplicationHistoryByMonitorChanged(String, Vec<ApplicationHistoryEntry>),
}

#[derive(Debug, Getters)]
pub struct ApplicationHistory {
    #[getset(get = "pub")]
    history: Vec<ApplicationHistoryEntry>,
    #[getset(get = "pub")]
    history_by_monitor: HashMap<String, Vec<ApplicationHistoryEntry>>,
    #[getset(get = "pub")]
    capacity: usize,
}

unsafe impl Send for ApplicationHistory {}
unsafe impl Send for ApplicationHistoryEvent {}

event_manager!(ApplicationHistory, ApplicationHistoryEvent);

impl ApplicationHistory {
    pub fn new() -> Result<Self, AppError> {
        Ok(ApplicationHistory {
            history: vec![],
            history_by_monitor: HashMap::new(),
            capacity: 100,
        })
    }

    pub fn current(&self) -> Result<&ApplicationHistoryEntry, AppError> {
        if self.history.is_empty() {
            Err("No history item found!".into())
        } else {
            Ok(&self.history[0])
        }
    }
    fn add(&mut self, item: ApplicationHistoryEntry) -> Result<(), AppError> {
        let application = item.application.clone();

        if self.history.is_empty() {
            self.history.push(item);
        } else {
            if item.application.hwnd == self.history[0].application.hwnd {
                return Ok(());
            }

            self.history.insert(0, item);

            if self.history.len() > self.capacity {
                let _ = self.history.pop();
            }
        }

        self.emit_event(Some(application));

        Ok(())
    }
    fn emit_event(&mut self, application: Option<FocusedApp>) {
        let sender = Self::event_tx();
        if let Some(application) = application {
            log_error!(
                sender.send(ApplicationHistoryEvent::ApplicationHistoryAdded(
                    application
                ))
            );
        }
        log_error!(sender.send(ApplicationHistoryEvent::ApplicationHistoryChanged));
        let current_state = self.get_filtered_by_monitor().unwrap();
        if self.history_by_monitor != current_state {
            let previous_state = self.history_by_monitor.clone();
            self.history_by_monitor = current_state.clone();

            for (monitor, items) in current_state {
                if !previous_state.contains_key(&monitor) || items != previous_state[&monitor] {
                    log_error!(sender.send(
                        ApplicationHistoryEvent::ApplicationHistoryByMonitorChanged(monitor, items,)
                    ));
                }
            }
        }
    }

    pub fn last_not_seelen_active(&self) -> Result<&ApplicationHistoryEntry, AppError> {
        self.history
            .iter()
            .find(|item| !item.is_seelen)
            .ok_or("Not matching history entry".into())
    }

    pub fn add_focused(&mut self, application: FocusedApp) -> Result<(), AppError> {
        let window = Window::from(application.hwnd);

        self.add(ApplicationHistoryEntry {
            application: application.clone(),
            focus_date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Wrong time")
                .as_millis() as u64,
            is_seelen: window.is_seelen_overlay(),
            on_monitor: window.monitor().device_id()?,
        })
    }

    pub fn update_current(&mut self, title: String) -> Result<(), AppError> {
        let item = if self.history.is_empty() {
            return Err("No history item found to update!".into());
        } else {
            &mut self.history[0]
        };

        item.application.title = title;

        let sender = Self::event_tx();
        log_error!(sender.send(ApplicationHistoryEvent::CurrentItemModified(
            item.application.clone()
        )));

        Ok(())
    }

    pub fn set_limit(&mut self, limit: usize) -> Result<(), AppError> {
        let mut modified = false;
        if self.capacity < limit {
            let (current, _) = self.history.split_at(limit);

            self.history = current.to_vec();
            modified = true;
        }

        self.capacity = limit;

        if modified {
            self.emit_event(None);
        }

        Ok(())
    }

    pub fn get_filtered_by_monitor(
        &self,
    ) -> Result<HashMap<String, Vec<ApplicationHistoryEntry>>, AppError> {
        let mut result = HashMap::new();

        for monitor in MonitorEnumerator::get_all_v2()? {
            let device_id = monitor.device_id()?;
            result.insert(
                device_id.clone(),
                self.history
                    .iter()
                    .filter(|item| item.on_monitor == device_id)
                    .cloned()
                    .collect(),
            );
        }

        Ok(result)
    }
}
