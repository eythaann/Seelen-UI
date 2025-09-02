use std::{
    sync::{Arc, LazyLock},
    time::Instant,
};

use parking_lot::Mutex;
use seelen_core::system_state::{Battery, PowerMode, PowerStatus};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Power::{
            PowerRegisterForEffectivePowerModeNotifications, PowerSettingRegisterNotification,
            PowerUnregisterFromEffectivePowerModeNotifications, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS,
            EFFECTIVE_POWER_MODE, EFFECTIVE_POWER_MODE_V2, HPOWERNOTIFY, SYSTEM_POWER_STATUS,
        },
        SystemServices::GUID_BATTERY_PERCENTAGE_REMAINING,
    },
    UI::WindowsAndMessaging::{
        DEVICE_NOTIFY_CALLBACK, PBT_APMPOWERSTATUSCHANGE, PBT_APMRESUMEAUTOMATIC,
        PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, WM_POWERBROADCAST,
    },
};

use crate::{
    app::get_app_handle,
    error::Result,
    event_manager,
    modules::power::domain::power_mode_to_serializable,
    trace_lock,
    windows_api::{event_window::subscribe_to_background_window, WindowsApi},
};

use super::domain::{battery_to_slu_battery, power_status_to_serializable};

pub static POWER_MANAGER: LazyLock<Arc<Mutex<PowerManager>>> = LazyLock::new(|| {
    let pm = match PowerManager::try_create_instance() {
        Ok(pm) => pm,
        Err(err) => {
            log::error!("Failed to create PowerManager instance: {err}");
            PowerManager::default()
        }
    };
    Arc::new(Mutex::new(pm))
});

#[derive(Debug)]
pub struct PowerManager {
    pub power_status: PowerStatus,
    pub current_power_mode: PowerMode,
    pub batteries: Vec<Battery>,

    pub last_suspend: Option<Instant>,
    power_setting_battery_percent_token: Option<HPOWERNOTIFY>,
    power_mode_event_token: Option<isize>,
}

event_manager!(PowerManager, PowerManagerEvent);

impl PowerManager {
    fn try_create_instance() -> Result<Self> {
        let mut instance = Self {
            power_status: Self::get_power_status()?,
            current_power_mode: PowerMode::Unknown,
            ..Default::default()
        };

        // https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powerregisterforeffectivepowermodenotifications#remarks
        unsafe {
            let mut unregister_token_ptr = std::ptr::null_mut();
            // Immediately after registration, the callback will be invoked with the current value of the power setting.
            PowerRegisterForEffectivePowerModeNotifications(
                EFFECTIVE_POWER_MODE_V2,
                Some(Self::on_effective_power_mode_change),
                None,
                &mut unregister_token_ptr,
            )?;
            instance.power_mode_event_token = Some(unregister_token_ptr as isize);
        }

        // https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powersettingregisternotification#remarks
        unsafe {
            let params = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
                Callback: Some(Self::on_battery_percent_change),
                ..Default::default()
            };

            let mut unregister_token_ptr = std::ptr::null_mut();
            // Immediately after registration, the callback will be invoked with the current value of the power setting.
            PowerSettingRegisterNotification(
                &GUID_BATTERY_PERCENTAGE_REMAINING,
                DEVICE_NOTIFY_CALLBACK,
                HANDLE(&params as *const _ as _),
                &mut unregister_token_ptr,
            )
            .ok()?;
            instance.power_setting_battery_percent_token =
                Some(HPOWERNOTIFY(unregister_token_ptr as isize));
        }

        subscribe_to_background_window(Self::on_bg_window_proc);
        Ok(instance)
    }

    unsafe extern "system" fn on_effective_power_mode_change(
        mode: EFFECTIVE_POWER_MODE,
        _ctx: *const std::ffi::c_void,
    ) {
        let mut guard = trace_lock!(POWER_MANAGER);
        let mode: PowerMode = power_mode_to_serializable(mode);
        if guard.current_power_mode != mode {
            log::trace!("Power mode changed to {mode:?}");
            guard.current_power_mode = mode;
            Self::send(PowerManagerEvent::PowerModeChanged(mode));
        }
    }

    unsafe extern "system" fn on_battery_percent_change(
        _context: *const std::ffi::c_void,
        _type: u32,
        _setting: *const std::ffi::c_void,
    ) -> u32 {
        if let Ok(batteries) = Self::get_batteries() {
            Self::send(PowerManagerEvent::BatteriesChanged(batteries));
        }
        0
    }

    pub fn get_power_status() -> Result<PowerStatus> {
        Ok(power_status_to_serializable(
            WindowsApi::get_system_power_status()?,
        ))
    }

    pub fn get_batteries() -> Result<Vec<Battery>> {
        let mut batteries: Vec<Battery> = Vec::new();
        let manager = battery::Manager::new()?;
        for battery in manager.batteries()?.flatten() {
            batteries.push(battery_to_slu_battery(battery)?);
        }
        Ok(batteries)
    }

    fn on_bg_window_proc(msg: u32, w_param: usize, _l_param: isize) -> Result<()> {
        if msg != WM_POWERBROADCAST {
            return Ok(());
        }

        match w_param as u32 {
            PBT_APMPOWERSTATUSCHANGE => {
                let mut guard = trace_lock!(POWER_MANAGER);
                let new_status = Self::get_power_status()?;
                if guard.power_status.ac_line_status != new_status.ac_line_status {
                    let batteries = Self::get_batteries()?;
                    guard.batteries = batteries.clone();
                    Self::send(PowerManagerEvent::BatteriesChanged(batteries));
                }
                log::trace!("Power status changed to {new_status:?}");
                guard.power_status = new_status.clone();
                Self::send(PowerManagerEvent::PowerStatusChanged(new_status));
            }
            PBT_APMSUSPEND => {
                log::info!("System suspended");
                trace_lock!(POWER_MANAGER).last_suspend = Some(Instant::now());
            }
            PBT_APMRESUMESUSPEND => {
                log::info!("System resumed (PBT_APMRESUMESUSPEND)");
            }
            PBT_APMRESUMEAUTOMATIC => {
                let last_suspend = trace_lock!(POWER_MANAGER).last_suspend.take();
                let elapsed = last_suspend.unwrap_or_else(Instant::now).elapsed();
                log::info!(
                    "System resumed (PBT_APMRESUMEAUTOMATIC) after {}s",
                    elapsed.as_secs()
                );
                // Always restart the app after wake up event
                get_app_handle().request_restart();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn release(&mut self) {
        if let Some(token) = self.power_mode_event_token.take() {
            let _ = unsafe { PowerUnregisterFromEffectivePowerModeNotifications(token as _) };
        }
        self.power_setting_battery_percent_token = None;
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self {
            power_status: power_status_to_serializable(SYSTEM_POWER_STATUS::default()),
            current_power_mode: PowerMode::Unknown,
            batteries: Vec::new(),
            last_suspend: None,
            power_mode_event_token: None,
            power_setting_battery_percent_token: None,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum PowerManagerEvent {
    PowerStatusChanged(PowerStatus),
    BatteriesChanged(Vec<Battery>),
    PowerModeChanged(PowerMode),
}
