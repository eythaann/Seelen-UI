use windows::{Foundation::TypedEventHandler, UI::Shell::FocusSessionManager};
use windows_core::IInspectable;

use crate::{
    error::{Result, ResultLogExt},
    system_modules::focus_assist::domain::{FocusAssistEvent, FocusAssistSystem},
};

pub struct WindowsFocusAssist {
    /// `None` until `init` acquires it.
    session: Option<FocusSessionManager>,
    active_changed_token: Option<i64>,
}

impl WindowsFocusAssist {
    fn session(&self) -> Result<&FocusSessionManager> {
        Ok(self
            .session
            .as_ref()
            .ok_or("FocusSessionManager is not initialized")?)
    }
}

impl FocusAssistSystem for WindowsFocusAssist {
    fn create() -> Self {
        Self {
            session: None,
            active_changed_token: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(FocusAssistEvent) + Send + Sync + 'static,
    {
        let session = self.session.insert(FocusSessionManager::GetDefault()?);
        let handler =
            TypedEventHandler::<FocusSessionManager, IInspectable>::new(move |sender, _args| {
                if let Some(session) = sender.as_ref() {
                    on_event(FocusAssistEvent::ActiveChanged {
                        active: session.IsFocusActive()?,
                    });
                }
                Ok(())
            });
        self.active_changed_token = Some(session.IsFocusActiveChanged(&handler)?);
        Ok(())
    }

    fn is_active(&self) -> Result<bool> {
        Ok(self.session()?.IsFocusActive()?)
    }

    fn set_active(&self, _enabled: bool) -> Result<()> {
        // https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.limitedaccessfeatures?view=winrt-28000
        // changing FocusAssistMode needs a Limited Access Feature access token, thing that doesn't work outside msix packages
        std::process::Command::new("explorer.exe")
            .arg("ms-settings:quiethours")
            .spawn()?;
        Ok(())
    }
}

impl Drop for WindowsFocusAssist {
    fn drop(&mut self) {
        if let (Some(session), Some(token)) = (&self.session, self.active_changed_token.take()) {
            session.RemoveIsFocusActiveChanged(token).log_error();
        }
    }
}
