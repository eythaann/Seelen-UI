use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::{mpsc::Sender, Arc};
use windows::{
    core::PCWSTR,
    Win32::{
        Devices::Display::GUID_DEVINTERFACE_MONITOR,
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::HMONITOR,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
            RegisterDeviceNotificationW, TranslateMessage, DBT_DEVTYP_DEVICEINTERFACE,
            DEVICE_NOTIFY_WINDOW_HANDLE, DEV_BROADCAST_DEVICEINTERFACE_W, MSG, WINDOW_EX_STYLE,
            WINDOW_STYLE, WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_SETTINGCHANGE, WNDCLASSW,
        },
    },
};

use crate::{
    error_handler::Result,
    log_error, trace_lock,
    utils::spawn_named_thread,
    windows_api::{MonitorEnumerator, WindowsApi},
};

lazy_static! {
    pub static ref MONITOR_MANAGER: Arc<Mutex<MonitorManager>> = Arc::new(Mutex::new(
        MonitorManager::new().expect("Failed to create monitor manager")
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorManagerEvent {
    Added(String, HMONITOR),
    Removed(String, HMONITOR),
    Updated(String, HMONITOR),
}

unsafe impl Send for MonitorManagerEvent {}

pub struct MonitorManager {
    pub monitors: Vec<(String, HMONITOR)>,
    callbacks: Vec<Sender<MonitorManagerEvent>>,
}

unsafe impl Send for MonitorManager {}

impl MonitorManager {
    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            // Added based on this https://stackoverflow.com/a/33762334
            WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
                // log::debug!("Dispatching {}, {:?}, {:?}", message, wparam, lparam);
                std::thread::spawn(move || {
                    let mut manager = trace_lock!(MONITOR_MANAGER);

                    let mut old_list = manager.monitors.clone();
                    let new_list = match Self::get_monitors() {
                        Ok(monitors) => monitors,
                        Err(_) => return,
                    };

                    for (name, id) in &new_list {
                        match old_list.iter().position(|x| x.0 == *name) {
                            Some(idx) => {
                                _ = old_list.remove(idx);
                                manager.notify_changes(MonitorManagerEvent::Updated(
                                    name.clone(),
                                    *id,
                                ));
                            }
                            None => {
                                manager
                                    .notify_changes(MonitorManagerEvent::Added(name.clone(), *id));
                            }
                        }
                    }

                    for (name, id) in old_list {
                        manager.notify_changes(MonitorManagerEvent::Removed(name, id));
                    }

                    manager.monitors = new_list.into_iter().collect();
                });
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }

    unsafe fn create_background_window() -> Result<()> {
        let wide_name: Vec<u16> = "Seelen Monitor Manager"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let wide_class: Vec<u16> = "SeelenMonitorManager"
            .encode_utf16()
            .chain(Some(0))
            .collect();

        let h_module = WindowsApi::module_handle_w()?;

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(Self::window_proc),
            hInstance: h_module.into(),
            lpszClassName: PCWSTR(wide_class.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

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

        let mut notification_filter = DEV_BROADCAST_DEVICEINTERFACE_W {
            dbcc_size: std::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
            dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE.0,
            dbcc_reserved: 0,
            dbcc_classguid: GUID_DEVINTERFACE_MONITOR,
            dbcc_name: [0; 1],
        };

        RegisterDeviceNotificationW(
            hwnd,
            &mut notification_filter as *mut _ as *mut _,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        )?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }

    pub fn new() -> Result<Self> {
        spawn_named_thread("Monitor Manager", || unsafe {
            log_error!(Self::create_background_window());
        })?;

        Ok(Self {
            callbacks: Vec::new(),
            monitors: Self::get_monitors()?,
        })
    }

    fn get_monitors() -> Result<Vec<(String, HMONITOR)>> {
        let mut monitors = Vec::new();
        for m in MonitorEnumerator::get_all()? {
            monitors.push((WindowsApi::monitor_name(m)?, m));
        }
        Ok(monitors)
    }

    pub fn listen_changes(&mut self, sender: Sender<MonitorManagerEvent>) {
        self.callbacks.push(sender);
    }

    pub fn notify_changes(&self, event: MonitorManagerEvent) {
        for callback in &self.callbacks {
            callback.send(event.clone()).ok();
        }
    }
}
