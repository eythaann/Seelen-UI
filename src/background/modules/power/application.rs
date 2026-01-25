use std::sync::{Arc, LazyLock};

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
        PBT_APMRESUMESUSPEND, WM_POWERBROADCAST,
    },
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::power::domain::power_mode_to_serializable,
    trace_lock,
    windows_api::{event_window::subscribe_to_background_window, WindowsApi},
};

use super::domain::{battery_to_slu_battery, power_status_to_serializable};

static POWER_MANAGER: LazyLock<Arc<Mutex<PowerManager>>> = LazyLock::new(|| {
    let mut pm = PowerManager::new();
    pm.init().log_error();
    Arc::new(Mutex::new(pm))
});

#[derive(Debug)]
pub struct PowerManager {
    pub power_status: PowerStatus,
    pub current_power_mode: PowerMode,
    pub batteries: Vec<Battery>,

    power_setting_battery_percent_token: Option<HPOWERNOTIFY>,
    power_mode_event_token: Option<isize>,
}

event_manager!(PowerManager, PowerManagerEvent);

impl PowerManager {
    pub fn instance() -> &'static Arc<Mutex<Self>> {
        &POWER_MANAGER
    }

    fn new() -> Self {
        Self::default()
    }

    fn init(&mut self) -> Result<()> {
        // Get initial power status and batteries
        self.power_status = Self::get_power_status()?;
        self.batteries = Self::get_batteries()?;

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
            self.power_mode_event_token = Some(unregister_token_ptr as isize);
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
            self.power_setting_battery_percent_token =
                Some(HPOWERNOTIFY(unregister_token_ptr as isize));
        }

        subscribe_to_background_window(Self::on_bg_window_proc);
        Ok(())
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
            PBT_APMRESUMESUSPEND | PBT_APMRESUMEAUTOMATIC => {
                log::trace!("System resuming from sleep, scheduling state refresh in 2 seconds");
                // Spawn a task to refresh state after 2 seconds
                // This is necessary because the power state may be stale immediately after wake up
                tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    log::trace!("Refreshing power state after wake up");
                    if let Ok(new_status) = Self::get_power_status() {
                        let mut guard = trace_lock!(POWER_MANAGER);
                        guard.power_status = new_status.clone();
                        Self::send(PowerManagerEvent::PowerStatusChanged(new_status));
                    }

                    if let Ok(batteries) = Self::get_batteries() {
                        let mut guard = trace_lock!(POWER_MANAGER);
                        guard.batteries = batteries.clone();
                        Self::send(PowerManagerEvent::BatteriesChanged(batteries));
                    }

                    let guard = trace_lock!(POWER_MANAGER);
                    let current_mode = guard.current_power_mode;
                    Self::send(PowerManagerEvent::PowerModeChanged(current_mode));
                });
            }
            _ => {}
        }

        Ok(())
    }

    #[allow(dead_code)]
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
