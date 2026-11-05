mod device;
mod state;

use seelen_core::system_state::{RadioDevice, RadioDeviceKind};

use crate::{
    error::Result,
    system_modules::radios::domain::{RadiosSysEvent, RadiosSystem},
};

use self::state::{RadiosState, RawRadioEvent};

/// Facade over [`RadiosState`], the singleton owning the radios watchers.
pub struct WindowsRadios {
    subscription: Option<String>,
}

impl RadiosSystem for WindowsRadios {
    fn create() -> Self {
        Self { subscription: None }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(RadiosSysEvent) + Send + Sync + 'static,
    {
        RadiosState::instance();
        self.subscription = Some(RadiosState::subscribe(move |event| match event {
            RawRadioEvent::Changed => on_event(RadiosSysEvent::Changed),
            RawRadioEvent::Enabled(kind) => on_event(RadiosSysEvent::Enabled { kind }),
        }));
        Ok(())
    }

    fn radios(&self) -> Vec<RadioDevice> {
        RadiosState::instance().get_radios()
    }

    fn set_enabled(&self, kind: RadioDeviceKind, enabled: bool) -> Result<()> {
        RadiosState::instance().set_radios_state(kind, enabled)
    }
}

impl Drop for WindowsRadios {
    fn drop(&mut self) {
        if let Some(id) = self.subscription.take() {
            RadiosState::unsubscribe(&id);
        }
    }
}
