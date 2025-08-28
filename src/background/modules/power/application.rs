use std::{sync::Arc, time::Instant};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Battery, PowerMode, PowerStatus},
};
use tauri::Emitter;
use windows::Win32::{
    System::Power::{
        PowerRegisterForEffectivePowerModeNotifications,
        PowerUnregisterFromEffectivePowerModeNotifications, EFFECTIVE_POWER_MODE,
        EFFECTIVE_POWER_MODE_V2,
    },
    UI::WindowsAndMessaging::{
        PBT_APMPOWERSTATUSCHANGE, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
        WM_POWERBROADCAST,
    },
};

use crate::{
    app::get_app_handle,
    error::Result,
    event_manager, log_error,
    modules::power::domain::power_mode_to_serializable,
    trace_lock,
    utils::spawn_named_thread,
    windows_api::{event_window::subscribe_to_background_window, WindowsApi},
};

use super::domain::{battery_to_slu_battery, power_status_to_serializable};

lazy_static! {
    pub static ref POWER_MANAGER: Arc<Mutex<PowerManager>> =
        Arc::new(Mutex::new(PowerManager::default()));
}

event_manager!(PowerManager, PowerManagerEvent);

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum PowerManagerEvent {
    PowerStatusChanged(PowerStatus),
    BatteriesChanged(Vec<Battery>),
    PowerModeChanged(PowerMode),
}

#[derive(Debug, Default)]
pub struct PowerManager {
    pub current_power_mode: Option<PowerMode>,
    pub last_suspend: Option<Instant>,
    power_mode_event_token: Option<isize>,
}

impl PowerManager {
    unsafe extern "system" fn on_effective_power_mode_change(
        mode: EFFECTIVE_POWER_MODE,
        _ctx: *const std::ffi::c_void,
    ) {
        let state: PowerMode = power_mode_to_serializable(mode);
        {
            trace_lock!(POWER_MANAGER).current_power_mode = Some(state.clone())
        }
        log_error!(Self::event_tx().send(PowerManagerEvent::PowerModeChanged(state)));
    }

    pub fn init(&mut self) -> Result<()> {
        Self::subscribe(|e| {
            let manager = trace_lock!(POWER_MANAGER);
            log_error!(manager.process_event(e))
        });

        unsafe {
            let mut unregister_token_ptr = std::ptr::null_mut();
            // will fail before windows 10 version 10.0.18363 (1909).
            // On first call it immediatelly callback with the current state to the registered callback!
            PowerRegisterForEffectivePowerModeNotifications(
                EFFECTIVE_POWER_MODE_V2,
                Some(Self::on_effective_power_mode_change),
                None,
                &mut unregister_token_ptr,
            )?;
            self.power_mode_event_token = Some(unregister_token_ptr as isize);
        }

        subscribe_to_background_window(Self::on_bg_window_proc);

        // TODO search for a better way to do this, WM_POWERBROADCAST only register status events
        // like charging, discharging, battery low, etc. not battery percentage change
        spawn_named_thread("Batery Refresh", move || loop {
            if let Ok(batteries) = Self::get_batteries() {
                log_error!(Self::event_tx().send(PowerManagerEvent::BatteriesChanged(batteries)));
            }
            std::thread::sleep(std::time::Duration::from_secs(60));
        })?;
        Ok(())
    }

    pub fn release(&self) -> Result<()> {
        if let Some(token) = self.power_mode_event_token {
            unsafe { PowerUnregisterFromEffectivePowerModeNotifications(token as _) }?;
        }
        Ok(())
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
        if msg == WM_POWERBROADCAST {
            match w_param as u32 {
                PBT_APMPOWERSTATUSCHANGE => {
                    Self::event_tx().send(PowerManagerEvent::PowerStatusChanged(
                        power_status_to_serializable(WindowsApi::get_system_power_status()?),
                    ))?;
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
        }
        Ok(())
    }

    fn process_event(&self, event: PowerManagerEvent) -> Result<()> {
        match event {
            PowerManagerEvent::PowerStatusChanged(status) => {
                let handle = get_app_handle();
                handle.emit(SeelenEvent::PowerStatus, status)?;
            }
            PowerManagerEvent::BatteriesChanged(batteries) => {
                let handle = get_app_handle();
                handle.emit(SeelenEvent::BatteriesStatus, batteries)?;
            }
            PowerManagerEvent::PowerModeChanged(mode) => {
                let handle = get_app_handle();
                handle.emit(SeelenEvent::PowerMode, mode)?;
            }
        }
        Ok(())
    }
}
