#[cfg(windows)]
mod windows;

use std::sync::{LazyLock, atomic::Ordering};

use seelen_core::state::shortcuts::ResolvedShortcut;
use slu_ipc::{AppIpc, messages::AppMessage};

use crate::{
    error::Result, exit, get_async_handler, log_error, modules::app_management::AppManager,
};

use super::domain::{HotkeysSystem, RecordingEvent, ShortcutAction};

#[cfg(windows)]
type Platform = windows::WindowsHotkeys;

pub struct HotkeysManager {
    platform: Platform,
}

impl HotkeysManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<HotkeysManager> = LazyLock::new(HotkeysManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    /// Replaces the registered shortcuts, an empty list stops them all.
    pub fn set_shortcuts(&self, shortcuts: Vec<ResolvedShortcut>) -> Result<()> {
        if shortcuts.is_empty() {
            self.stop();
            return Ok(());
        }
        self.platform.register(shortcuts, Self::on_shortcut)
    }

    pub fn stop(&self) {
        self.platform.stop();
    }

    pub async fn start_recording(&self) -> Result<()> {
        send_recording_to_app(Some(vec![])).await?;
        self.platform.start_recording(|event| {
            let keys = match event {
                RecordingEvent::Pressed { keys } => Some(keys),
                RecordingEvent::Finished => None,
            };
            get_async_handler().spawn(async move {
                let _ = send_recording_to_app(keys).await;
            });
        })
    }

    pub fn stop_recording(&self) {
        self.platform.stop_recording();
    }

    fn on_shortcut(command: &[String]) {
        log::trace!("Hotkey triggered: {command:?}");
        match ShortcutAction::of(command) {
            ShortcutAction::RestartApp => {
                log_error!(AppManager::instance().kill_app_instances());
            }
            ShortcutAction::Quit => {
                crate::EXITING.store(true, Ordering::SeqCst);
                log_error!(AppManager::instance().kill_app_instances());
                exit(0);
            }
            ShortcutAction::ForwardToApp => {
                let command = command.to_vec();
                get_async_handler().spawn(async move {
                    log_error!(AppIpc::send(AppMessage::Cli(command)).await);
                });
            }
        }
    }
}

/// `None` tells the app that the recording finished.
async fn send_recording_to_app(keys: Option<Vec<String>>) -> Result<()> {
    AppIpc::send(AppMessage::Cli(vec![
        "popup".to_owned(),
        "internal-set-shortcut".to_owned(),
        serde_json::to_string(&keys)?,
    ]))
    .await?;
    Ok(())
}
