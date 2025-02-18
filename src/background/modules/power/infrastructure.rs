use std::{
    ffi::c_void,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};

use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{
            Power::{
                PowerRegisterForEffectivePowerModeNotifications,
                PowerUnregisterFromEffectivePowerModeNotifications, EFFECTIVE_POWER_MODE,
                EFFECTIVE_POWER_MODE_V1, EFFECTIVE_POWER_MODE_V2,
            },
            Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
            RegisterClassW, TranslateMessage, MSG, PBT_APMPOWERSTATUSCHANGE, WINDOW_EX_STYLE,
            WINDOW_STYLE, WM_DESTROY, WM_POWERBROADCAST, WNDCLASSW,
        },
    },
};

use crate::{
    error_handler::Result,
    log_error,
    modules::power::domain::{Battery, PowerPlan},
    seelen::get_app_handle,
    utils::spawn_named_thread,
    windows_api::WindowsApi,
};

use super::domain::PowerStatus;

static REGISTERED: AtomicBool = AtomicBool::new(false);
static REGISTRATION_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub fn release_power_events() {
    let handle = REGISTRATION_HANDLE.load(Ordering::Acquire);
    let result =
        unsafe { PowerUnregisterFromEffectivePowerModeNotifications(handle as *const c_void) };
    log_error!(result);
}

pub struct PowerManager;
impl PowerManager {
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_POWERBROADCAST => {
                if PBT_APMPOWERSTATUSCHANGE == w_param.0 as u32 {
                    log_error!(PowerManager::emit_system_power_info());
                }
                LRESULT(1)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
    unsafe extern "system" fn on_effective_power_mode_change(
        mode: EFFECTIVE_POWER_MODE,
        _ctx: *const c_void,
    ) {
        let power_mode: Result<PowerPlan> = mode.try_into();
        if let Ok(power_mode) = power_mode {
            log_error!(get_app_handle().emit(SeelenEvent::PowerPlan, power_mode));
        } else {
            log_error!(power_mode);
        }
    }

    pub fn register_power_events() -> Result<()> {
        if REGISTERED.load(Ordering::Acquire) {
            return Ok(());
        }
        REGISTERED.store(true, Ordering::Release);
        log::trace!("Registering system power events");

        let wide_name: Vec<u16> = "Seelen Power Manager"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let wide_class: Vec<u16> = "SeelenPowerManager".encode_utf16().chain(Some(0)).collect();

        let h_module = WindowsApi::module_handle_w()?;

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(Self::window_proc),
            hInstance: h_module.into(),
            lpszClassName: PCWSTR(wide_class.as_ptr()),
            ..Default::default()
        };

        unsafe {
            RegisterClassW(&wnd_class);
        }

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(wide_class.as_ptr()),
                PCWSTR(wide_name.as_ptr()),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                h_module,
                None,
            )?
        };

        let addr = hwnd.0 as isize;
        spawn_named_thread("Power Manager Message Loop", move || unsafe {
            let hwnd = HWND(addr as _);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, hwnd, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        })?;

        let mut unregister: isize = 0;
        // https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powerregisterforeffectivepowermodenotifications
        unsafe {
            if PowerRegisterForEffectivePowerModeNotifications(
                EFFECTIVE_POWER_MODE_V2,
                Some(Self::on_effective_power_mode_change),
                None,
                &mut unregister as *mut isize as *mut *mut c_void,
            )
            .is_err()
            {
                // Awaited error in case V2 not supported before Windows 10, version 1903
                PowerRegisterForEffectivePowerModeNotifications(
                    EFFECTIVE_POWER_MODE_V1,
                    Some(Self::on_effective_power_mode_change),
                    None,
                    &mut unregister as *mut isize as *mut *mut c_void,
                )?;
            }

            REGISTRATION_HANDLE.store(unregister, Ordering::Release);
        };

        // TODO search for a better way to do this, WM_POWERBROADCAST only register status events
        // like charging, discharging, battery low, etc.
        spawn_named_thread("Power Manager Loop", move || loop {
            log_error!(PowerManager::emit_system_power_info());
            std::thread::sleep(std::time::Duration::from_secs(60));
        })?;
        Ok(())
    }

    pub fn emit_system_power_info() -> Result<()> {
        let handle = get_app_handle();

        let power_status: PowerStatus = WindowsApi::get_system_power_status()?.into();
        handle.emit(SeelenEvent::PowerStatus, power_status)?;

        let mut batteries: Vec<Battery> = Vec::new();
        let manager = battery::Manager::new()?;
        for battery in manager.batteries()?.flatten() {
            batteries.push(battery.try_into()?);
        }

        handle.emit(SeelenEvent::BatteriesStatus, batteries)?;

        Ok(())
    }
}

#[tauri::command(async)]
pub fn log_out() {
    log_error!(WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE));
}

#[tauri::command(async)]
pub fn suspend() {
    log_error!(WindowsApi::set_suspend_state());
}

#[tauri::command(async)]
pub fn restart() -> Result<()> {
    WindowsApi::exit_windows(EWX_REBOOT, SHTDN_REASON_NONE)?;
    Ok(())
}

#[tauri::command(async)]
pub fn shutdown() -> Result<()> {
    WindowsApi::exit_windows(EWX_SHUTDOWN, SHTDN_REASON_NONE)?;
    Ok(())
}

#[tauri::command(async)]
pub fn lock() -> Result<()> {
    WindowsApi::lock_machine()?;
    Ok(())
}
