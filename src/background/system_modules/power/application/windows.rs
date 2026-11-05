use std::{ffi::c_void, sync::Arc};

use seelen_core::system_state::{Battery, PowerMode, PowerStatus};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Power::{
            DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, EFFECTIVE_POWER_MODE, EFFECTIVE_POWER_MODE_V2,
            HPOWERNOTIFY, PowerRegisterForEffectivePowerModeNotifications,
            PowerSettingRegisterNotification, PowerSettingUnregisterNotification,
            PowerUnregisterFromEffectivePowerModeNotifications, SYSTEM_POWER_STATUS,
        },
        Shutdown::{
            EWX_LOGOFF, SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_OPERATINGSYSTEM,
            SHTDN_REASON_MINOR_UPGRADE, SHTDN_REASON_NONE, SHUTDOWN_FLAGS,
            SHUTDOWN_INSTALL_UPDATES, SHUTDOWN_POWEROFF, SHUTDOWN_REASON, SHUTDOWN_RESTART,
        },
        SystemServices::GUID_BATTERY_PERCENTAGE_REMAINING,
    },
    UI::WindowsAndMessaging::{
        DEVICE_NOTIFY_CALLBACK, PBT_APMPOWERSTATUSCHANGE, PBT_APMRESUMEAUTOMATIC,
        PBT_APMRESUMESUSPEND, WM_POWERBROADCAST,
    },
};
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

use crate::{
    error::{Result, ResultLogExt},
    system_modules::power::domain::{PowerSysEvent, PowerSystem, battery_from_device},
    windows_api::{WindowsApi, event_window::subscribe_to_background_window},
};

/// Keys created by Windows Update / CBS when updates are waiting for a shutdown or restart
/// to be applied (the same condition that makes the shell show "Update and shut down").
const PENDING_UPDATES_REGISTRY_KEYS: [&str; 2] = [
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
];

type EventCallback = Box<dyn Fn(PowerSysEvent) + Send + Sync>;

pub struct WindowsPower {
    /// Context of the native callbacks (`Arc::as_ptr`). It must outlive their registrations,
    /// which `Drop` releases before the fields are dropped.
    on_event: Option<Arc<EventCallback>>,
    /// Raw `PowerRegisterForEffectivePowerModeNotifications` handle.
    power_mode_registration: Option<isize>,
    battery_percent_registration: Option<HPOWERNOTIFY>,
}

impl PowerSystem for WindowsPower {
    fn create() -> Self {
        Self {
            on_event: None,
            power_mode_registration: None,
            battery_percent_registration: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(PowerSysEvent) + Send + Sync + 'static,
    {
        let on_event: Arc<EventCallback> = Arc::new(Box::new(on_event));
        let context = Arc::as_ptr(&on_event) as *const c_void;
        self.on_event = Some(on_event.clone());

        // https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powerregisterforeffectivepowermodenotifications#remarks
        // Immediately after registration, the callback is invoked with the current mode.
        let mut handle = std::ptr::null_mut();
        unsafe {
            PowerRegisterForEffectivePowerModeNotifications(
                EFFECTIVE_POWER_MODE_V2,
                Some(on_power_mode_change),
                Some(context),
                &mut handle,
            )?
        };
        self.power_mode_registration = Some(handle as isize);

        // https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powersettingregisternotification#remarks
        let params = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(on_battery_percent_change),
            Context: context as *mut c_void,
        };
        let mut handle = std::ptr::null_mut();
        unsafe {
            PowerSettingRegisterNotification(
                &GUID_BATTERY_PERCENTAGE_REMAINING,
                DEVICE_NOTIFY_CALLBACK,
                HANDLE(&params as *const _ as _),
                &mut handle,
            )
            .ok()?
        };
        self.battery_percent_registration = Some(HPOWERNOTIFY(handle as isize));

        subscribe_to_background_window(move |msg, w_param, _l_param| {
            if msg == WM_POWERBROADCAST {
                match w_param as u32 {
                    PBT_APMPOWERSTATUSCHANGE => on_event(PowerSysEvent::StatusChanged),
                    PBT_APMRESUMESUSPEND | PBT_APMRESUMEAUTOMATIC => {
                        on_event(PowerSysEvent::Resumed)
                    }
                    _ => {}
                }
            }
            Ok(())
        });
        Ok(())
    }

    fn power_status(&self) -> Result<PowerStatus> {
        Ok(power_status_from_native(
            WindowsApi::get_system_power_status()?,
        ))
    }

    fn batteries(&self) -> Result<Vec<Battery>> {
        let manager = battery::Manager::new()?;
        Ok(manager
            .batteries()?
            .flatten()
            .map(battery_from_device)
            .collect())
    }

    fn has_pending_os_updates(&self) -> bool {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        PENDING_UPDATES_REGISTRY_KEYS
            .iter()
            .any(|path| hklm.open_subkey(path).is_ok())
    }

    fn log_out(&self) -> Result<()> {
        WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE)
    }

    fn suspend(&self) -> Result<()> {
        WindowsApi::set_suspend_state(false)
    }

    fn hibernate(&self) -> Result<()> {
        WindowsApi::set_suspend_state(true)
    }

    fn lock(&self) -> Result<()> {
        WindowsApi::lock_machine()
    }

    fn restart(&self, install_updates: bool) -> Result<()> {
        initiate_shutdown(SHUTDOWN_RESTART, install_updates)
    }

    fn shutdown(&self, install_updates: bool) -> Result<()> {
        initiate_shutdown(SHUTDOWN_POWEROFF, install_updates)
    }
}

impl Drop for WindowsPower {
    fn drop(&mut self) {
        if let Some(handle) = self.power_mode_registration.take() {
            unsafe { PowerUnregisterFromEffectivePowerModeNotifications(handle as _) }.log_error();
        }
        if let Some(handle) = self.battery_percent_registration.take() {
            unsafe { PowerSettingUnregisterNotification(handle) }
                .ok()
                .log_error();
        }
    }
}

unsafe extern "system" fn on_power_mode_change(mode: EFFECTIVE_POWER_MODE, context: *const c_void) {
    // SAFETY: `context` points to the `EventCallback` kept alive by `WindowsPower::on_event`.
    let on_event = unsafe { &*(context as *const EventCallback) };
    on_event(PowerSysEvent::ModeChanged {
        mode: power_mode_from_native(mode),
    });
}

unsafe extern "system" fn on_battery_percent_change(
    context: *const c_void,
    _type: u32,
    _setting: *const c_void,
) -> u32 {
    // SAFETY: `context` points to the `EventCallback` kept alive by `WindowsPower::on_event`.
    let on_event = unsafe { &*(context as *const EventCallback) };
    on_event(PowerSysEvent::BatteriesChanged);
    0
}

/// Updates are only applied when explicitly requested, mirroring the shell's
/// "Shut down" vs "Update and shut down" options.
fn initiate_shutdown(mut flags: SHUTDOWN_FLAGS, install_updates: bool) -> Result<()> {
    let mut reason: SHUTDOWN_REASON = SHTDN_REASON_NONE;
    if install_updates {
        flags |= SHUTDOWN_INSTALL_UPDATES;
        reason = SHTDN_REASON_FLAG_PLANNED
            | SHTDN_REASON_MAJOR_OPERATINGSYSTEM
            | SHTDN_REASON_MINOR_UPGRADE;
    }
    WindowsApi::initiate_shutdown(flags, reason)
}

fn power_status_from_native(status: SYSTEM_POWER_STATUS) -> PowerStatus {
    PowerStatus {
        ac_line_status: status.ACLineStatus,
        battery_flag: status.BatteryFlag,
        battery_life_percent: status.BatteryLifePercent,
        system_status_flag: status.SystemStatusFlag,
        battery_life_time: status.BatteryLifeTime,
        battery_full_life_time: status.BatteryFullLifeTime,
    }
}

// https://learn.microsoft.com/en-us/windows/win32/api/powersetting/ne-powersetting-effective_power_mode
fn power_mode_from_native(mode: EFFECTIVE_POWER_MODE) -> PowerMode {
    match mode.0 {
        0 => PowerMode::BatterySaver,
        1 => PowerMode::BetterBattery,
        2 => PowerMode::Balanced,
        3 => PowerMode::HighPerformance,
        4 => PowerMode::MaxPerformance,
        5 => PowerMode::GameMode,
        6 => PowerMode::MixedReality,
        _ => PowerMode::Unknown,
    }
}
