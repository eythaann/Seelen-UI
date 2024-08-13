use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Emitter;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{FALSE, HWND, LPARAM, LRESULT, WPARAM},
        Security::{
            AdjustTokenPrivileges, SE_PRIVILEGE_ENABLED, SE_SHUTDOWN_NAME, TOKEN_PRIVILEGES,
        },
        System::Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
            RegisterClassW, TranslateMessage, MSG, PBT_APMPOWERSTATUSCHANGE, WINDOW_EX_STYLE,
            WINDOW_STYLE, WM_DESTROY, WM_POWERBROADCAST, WNDCLASSW,
        },
    },
};

use crate::{
    error_handler::Result, log_error, modules::power::domain::Battery, seelen::get_app_handle,
    windows_api::WindowsApi,
};

use super::domain::PowerStatus;

static REGISTERED: AtomicBool = AtomicBool::new(false);

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

    pub fn register_power_events() -> Result<()> {
        if REGISTERED.load(Ordering::Acquire) {
            return Ok(());
        }
        log::info!("Registering system power events");

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

        std::thread::spawn(move || unsafe {
            let hwnd = CreateWindowExW(
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
            );

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, hwnd, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });

        // TODO search for a better way to do this, WM_POWERBROADCAST only register status events
        // like charging, discharging, battery low, etc.
        std::thread::spawn(move || loop {
            log_error!(PowerManager::emit_system_power_info());
            std::thread::sleep(std::time::Duration::from_secs(60));
        });

        REGISTERED.store(true, Ordering::Release);
        Ok(())
    }

    pub fn emit_system_power_info() -> Result<()> {
        let handle = get_app_handle();

        let power_status: PowerStatus = WindowsApi::get_system_power_status()?.into();
        handle.emit("power-status", power_status)?;

        let mut batteries: Vec<Battery> = Vec::new();
        let manager = battery::Manager::new()?;
        for battery in manager.batteries()?.flatten() {
            batteries.push(battery.try_into()?);
        }

        handle.emit("batteries-status", batteries)?;

        Ok(())
    }
}

#[tauri::command]
pub fn log_out() {
    log_error!(WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE));
}

#[tauri::command]
pub fn suspend() {
    log_error!(WindowsApi::set_suspend_state());
}

#[tauri::command]
pub fn restart() -> Result<(), String> {
    let token_handle = WindowsApi::open_process_token()?;
    let mut tkp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        ..Default::default()
    };

    tkp.Privileges[0].Luid = WindowsApi::get_luid(PCWSTR::null(), SE_SHUTDOWN_NAME)?;
    tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    unsafe {
        AdjustTokenPrivileges(token_handle, FALSE, Some(&tkp), 0, None, None)
            .expect("Could not adjust token privileges");
    }

    WindowsApi::exit_windows(EWX_REBOOT, SHTDN_REASON_NONE)?;
    Ok(())
}

#[tauri::command]
pub fn shutdown() -> Result<(), String> {
    let token_handle = WindowsApi::open_process_token()?;
    let mut tkp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        ..Default::default()
    };

    tkp.Privileges[0].Luid = WindowsApi::get_luid(PCWSTR::null(), SE_SHUTDOWN_NAME)?;
    tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    unsafe {
        AdjustTokenPrivileges(token_handle, FALSE, Some(&tkp), 0, None, None)
            .expect("Could not adjust token privileges");
    }

    WindowsApi::exit_windows(EWX_SHUTDOWN, SHTDN_REASON_NONE)?;
    Ok(())
}
