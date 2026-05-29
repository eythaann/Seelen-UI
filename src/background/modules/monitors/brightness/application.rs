use std::sync::LazyLock;

use wmi::WMIConnection;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::monitors::brightness::domain::{
        WmiMonitorBrightness, WmiMonitorBrightnessEvent, WmiMonitorBrightnessMethods,
        WmiSetBrightnessPayload,
    },
    utils::lock_free::SyncVec,
};

#[derive(Debug, Clone)]
pub enum BrightnessManagerEvent {
    Changed(Vec<WmiMonitorBrightness>),
}

event_manager!(BrightnessManager, BrightnessManagerEvent);

pub struct BrightnessManager {
    brightness: SyncVec<WmiMonitorBrightness>,
}

impl BrightnessManager {
    pub fn instance() -> &'static Self {
        static INSTANCE: LazyLock<BrightnessManager> = LazyLock::new(|| {
            let mut m = BrightnessManager::new();
            m.init().log_error();
            m
        });
        &INSTANCE
    }

    fn new() -> Self {
        Self {
            brightness: SyncVec::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;

        std::thread::spawn(move || {
            let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
            // Most desktop/external monitors don't expose the WMI brightness class;
            // bail out quietly instead of logging an error when listening is unsupported.
            let notifications = match wmi.notification::<WmiMonitorBrightnessEvent>() {
                Ok(notifications) => notifications,
                Err(err) => {
                    log::debug!("Monitor brightness events are not available: {err:?}");
                    return Result::Ok(());
                }
            };
            for event in notifications {
                let Ok(_event) = event else {
                    continue;
                };

                let brightness: Vec<WmiMonitorBrightness> = wmi.query()?;
                BrightnessManager::instance()
                    .brightness
                    .replace(brightness.clone());
                BrightnessManager::send(BrightnessManagerEvent::Changed(brightness));
            }
            Result::Ok(())
        });

        // A display that doesn't support WMI brightness control (common on
        // desktops and external monitors) is treated as "no controllable
        // monitors" rather than an error, to avoid noisy error logs.
        match wmi.query::<WmiMonitorBrightness>() {
            Ok(list) => self.brightness = list.into(),
            Err(err) => log::debug!("Monitor brightness is not available: {err:?}"),
        }
        Ok(())
    }

    pub fn get_all_brightness(&self) -> Vec<WmiMonitorBrightness> {
        self.brightness.to_vec()
    }

    pub fn set_brightness(&self, instance_name: &str, level: u8) -> Result<()> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;

        let instances = wmi.query::<WmiMonitorBrightnessMethods>()?;

        let obj = instances
            .into_iter()
            .find(|v| v.instance_name == instance_name)
            .ok_or("Instance not found")?;

        wmi.exec_instance_method::<WmiMonitorBrightnessMethods, ()>(
            obj.__path,
            "WmiSetBrightness",
            WmiSetBrightnessPayload {
                timeout: 0,
                brightness: level,
            },
        )?;
        Ok(())
    }
}
