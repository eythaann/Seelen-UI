use std::sync::LazyLock;

use windows::{Foundation::TypedEventHandler, UI::Shell::FocusSessionManager};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

pub struct FocusAssistManager {
    winrt: FocusSessionManager,
    event_token: Option<i64>,
}

unsafe impl Send for FocusAssistManager {}
unsafe impl Sync for FocusAssistManager {}

event_manager!(FocusAssistManager, bool);

impl FocusAssistManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<FocusAssistManager> = LazyLock::new(|| {
            let mut m = FocusAssistManager::new().expect("Failed to create FocusAssistManager");
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn new() -> Result<Self> {
        let winrt = FocusSessionManager::GetDefault()?;
        Ok(Self {
            winrt,
            event_token: None,
        })
    }

    fn init(&mut self) -> Result<()> {
        let token = self
            .winrt
            .IsFocusActiveChanged(&TypedEventHandler::new(Self::on_focus_active_changed))?;
        self.event_token = Some(token);
        Ok(())
    }

    fn on_focus_active_changed(
        sender: windows_core::Ref<FocusSessionManager>,
        _args: windows_core::Ref<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        let Some(manager) = sender.as_ref() else {
            return Ok(());
        };
        Self::send(manager.IsFocusActive()?);
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.winrt.IsFocusActive().unwrap_or(false)
    }

    pub fn set_focus_assist(&self, _enabled: bool) -> Result<()> {
        // https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.limitedaccessfeatures?view=winrt-28000
        // changing FocusAssistMode needs a Limited Access Feature access token, thing that doesn't work outside msix packages
        std::process::Command::new("explorer.exe")
            .arg("ms-settings:quiethours")
            .spawn()?;
        Ok(())
    }
}

impl Drop for FocusAssistManager {
    fn drop(&mut self) {
        if let Some(token) = self.event_token.take() {
            self.winrt.RemoveIsFocusActiveChanged(token).log_error();
        }
    }
}
