use parking_lot::Mutex;
use seelen_core::system_state::{RegistryNotifyIcon, TrayIcon};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tauri::Listener;
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    System::DataExchange::COPYDATASTRUCT,
    UI::{
        Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
            IUIAutomationElement3, IUIAutomationInvokePattern, TreeScope, TreeScope_Children,
            TreeScope_Subtree, UIA_InvokePatternId,
        },
        Shell::{
            Shell_NotifyIconGetRect, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
            NOTIFYICONIDENTIFIER,
        },
        WindowsAndMessaging::{
            FindWindowA, FindWindowExA, GetCursorPos, RegisterWindowMessageW, SendNotifyMessageW,
            HWND_BROADCAST, SW_HIDE, SW_SHOW, WM_COPYDATA,
        },
    },
};
use windows_core::{Interface, GUID};
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS},
    RegKey,
};

use crate::{
    error_handler::Result,
    event_manager, pcstr,
    seelen::get_app_handle,
    trace_lock,
    utils::{is_windows_10, is_windows_11, resolve_guid_path, sleep_millis},
    windows_api::{
        event_window::{get_native_shell_hwnd, subscribe_to_background_window},
        string_utils::WindowsString,
        AppBarData, AppBarDataState, Com, WindowEnumerator, WindowsApi,
    },
};

use super::domain::{NotifyIconIdentifier, ShellTrayMessage, TrayIconId, TrayIconV2};

lazy_static! {
    pub static ref TRAY_ICON_MANAGER: Arc<Mutex<TrayIconManager>> =
        Arc::new(Mutex::new(TrayIconManager::new()));
}

pub struct TrayIconManager {
    pub icons: Vec<TrayIcon>,
    pub au_element_by_key: HashMap<String, TrayIconUIElement>,
    pub icons_v2: HashMap<TrayIconId, TrayIconV2>,
}

unsafe impl Send for TrayIconManager {}

#[derive(Debug, Clone)]
pub enum TrayIconEvent {
    Added(TrayIconV2),
    Updated(TrayIconV2),
    Removed(TrayIconId),
    ForceUpdate,
}

unsafe impl Send for TrayIconEvent {}

event_manager!(TrayIconManager, TrayIconEvent);

impl TrayIconManager {
    fn new() -> Self {
        Self {
            icons: Vec::new(),
            icons_v2: HashMap::new(),
            au_element_by_key: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        subscribe_to_background_window(Self::on_bg_window_proc);
        Self::subscribe(|event| match event {
            TrayIconEvent::Added(data) => {
                log::trace!("Tray icon added {}", data.id);
                let mut manager = trace_lock!(TRAY_ICON_MANAGER);
                manager.icons_v2.insert(data.id.clone(), data);
                if let Ok((icons, elements)) = manager.load_all_tray_icons() {
                    manager.icons = icons;
                    manager.au_element_by_key = elements;
                }
            }
            TrayIconEvent::Updated(data) => {
                log::trace!("Tray icon updated {}", data.id);
                let mut manager = trace_lock!(TRAY_ICON_MANAGER);
                manager.icons_v2.insert(data.id.clone(), data);
            }
            TrayIconEvent::Removed(id) => {
                log::trace!("Tray icon removed {id}");
                let mut manager = trace_lock!(TRAY_ICON_MANAGER);
                manager.icons_v2.remove(&id);
                if let Ok((icons, elements)) = manager.load_all_tray_icons() {
                    manager.icons = icons;
                    manager.au_element_by_key = elements;
                }
            }
            TrayIconEvent::ForceUpdate => {
                let mut manager = trace_lock!(TRAY_ICON_MANAGER);
                if let Ok((icons, elements)) = manager.load_all_tray_icons() {
                    manager.icons = icons;
                    manager.au_element_by_key = elements;
                }
            }
        });
        Self::refresh_icons()?;

        get_app_handle().listen("hidden::tray-force-refresh", |_| {
            let _ = Self::event_tx().send(TrayIconEvent::ForceUpdate);
        });

        let (icons, elements) = self.load_all_tray_icons()?;
        self.icons = icons;
        self.au_element_by_key = elements;
        Ok(())
    }

    fn on_bg_window_proc(msg: u32, _w_param: usize, l_param: isize) -> Result<()> {
        if msg == WM_COPYDATA {
            let data = match unsafe { (l_param as *const COPYDATASTRUCT).as_ref() } {
                Some(data) => data,
                None => return Ok(()),
            };

            match data.dwData {
                1 => {
                    let message = match unsafe { (data.lpData as *mut ShellTrayMessage).as_mut() } {
                        Some(message) => message,
                        None => return Ok(()),
                    };

                    let data: TrayIconV2 = message.data.into();

                    let event = match message.message {
                        NIM_ADD => Some(TrayIconEvent::Added(data)),
                        NIM_MODIFY | NIM_SETVERSION => Some(TrayIconEvent::Updated(data)),
                        NIM_DELETE => Some(TrayIconEvent::Removed(data.id)),
                        _ => None,
                    };

                    if let Some(event) = event {
                        Self::event_tx().send(event)?;
                    }
                }
                3 => {
                    let _message =
                        match unsafe { (data.lpData as *mut NotifyIconIdentifier).as_mut() } {
                            Some(message) => message,
                            None => return Ok(()),
                        };
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Refreshes the icons of the tray.
    ///
    /// Simulates the Windows taskbar being re-created. Some windows fail to
    /// re-add their icons, in which case it's an implementation error on
    /// their side. These windows that fail also do not re-add their icons
    /// to the Windows taskbar when `explorer.exe` is restarted ordinarily.
    pub fn refresh_icons() -> Result<()> {
        log::trace!("Refreshing icons by sending `TaskbarCreated` message.");
        let msg = WindowsString::from("TaskbarCreated");
        let msg = unsafe { RegisterWindowMessageW(msg.as_pcwstr()) };
        if msg == 0 {
            return Err(windows::core::Error::from_win32().into());
        }
        unsafe { SendNotifyMessageW(HWND_BROADCAST, msg, std::mem::zeroed(), std::mem::zeroed()) }?;
        Ok(())
    }

    pub fn load_all_tray_icons(
        &self,
    ) -> Result<(Vec<TrayIcon>, HashMap<String, TrayIconUIElement>)> {
        ensure_tray_overflow_creation()?;
        let tray_from_registry = Self::enum_from_registry()?;

        Com::run_with_context(|| unsafe {
            let mut tray_elements = Vec::new();
            let mut automation_by_key = HashMap::new();

            let automation: IUIAutomation = Com::create_instance(&CUIAutomation)?;
            let condition = automation.CreateTrueCondition()?;

            let mut children = Vec::new();
            if let Some(tray_overflow) = get_tray_overflow_content_handle() {
                let element: IUIAutomationElement = automation.ElementFromHandle(tray_overflow)?;
                children.extend(get_sub_tree(&element, &condition, TreeScope_Children)?);
            }

            let mut idx = 0;
            for element in children {
                // running items got from the registry should be the same amount as the ones in the tray overflow
                // and in the same order
                if let Some(item_on_reg) = tray_from_registry.get(idx) {
                    tray_elements.push(TrayIcon {
                        label: element
                            .CurrentName()
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                        registry: item_on_reg.clone(),
                    });
                    automation_by_key.insert(item_on_reg.key.clone(), TrayIconUIElement(element));
                    idx += 1;
                }
            }
            Ok((tray_elements, automation_by_key))
        })
    }

    fn send_all_to_tray_overflow() -> Result<()> {
        let hkcr = RegKey::predef(HKEY_CURRENT_USER);
        let settings =
            hkcr.open_subkey_with_flags("Control Panel\\NotifyIconSettings", KEY_ALL_ACCESS)?;
        let list = settings.get_raw_value("UIOrderList")?.bytes;
        for chunk in list.chunks(8) {
            let key = u64::from_le_bytes(chunk.try_into()?).to_string();
            let regkey = settings.open_subkey_with_flags(&key, KEY_ALL_ACCESS)?;
            regkey.set_value("IsPromoted", &0u32)?;
        }
        Ok(())
    }

    // TODO: remove this, instead hide sys tray module in case of disabled
    pub fn enable_chevron() -> Result<()> {
        let hkcr = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcr.open_subkey_with_flags(
            r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\TrayNotify",
            KEY_ALL_ACCESS,
        )?;
        settings.set_value("SystemTrayChevronVisibility", &1u32)?;
        Ok(())
    }

    pub fn enum_from_registry() -> Result<Vec<RegistryNotifyIcon>> {
        let hkcr = RegKey::predef(HKEY_CURRENT_USER);
        let settings =
            hkcr.open_subkey_with_flags("Control Panel\\NotifyIconSettings", KEY_ALL_ACCESS)?;

        // the order in this list is the order in which the icons will be displayed on the Win Taskbar and Win Tray Overflow
        let list = settings.get_raw_value("UIOrderList")?.bytes;

        let mut windows = Vec::new();
        WindowEnumerator::new().for_each(|w| {
            if let Ok(path) = w.process().program_path() {
                windows.push((w.address(), path.to_string_lossy().to_lowercase()));
            }
        })?;

        let mut registers = Vec::new();
        for chunk in list.chunks(8) {
            let key = u64::from_le_bytes(chunk.try_into()?).to_string();
            let regkey = settings.open_subkey_with_flags(&key, KEY_ALL_ACCESS)?;

            // custom promotion value, to be used on our side
            let promoted: u32 = regkey.get_value("IsSluPromoted").unwrap_or_default();

            let path_with_guid: String = regkey.get_value("ExecutablePath")?;
            let mut item = RegistryNotifyIcon {
                key,
                executable_path: resolve_guid_path(path_with_guid)?,
                icon_snapshot: regkey.get_raw_value("IconSnapShot").map(|v| v.bytes).ok(),
                initial_tooltip: regkey.get_value("InitialTooltip").ok(),
                icon_guid: regkey.get_value("IconGuid").ok(),
                icon_uid: regkey.get_value("UID").ok(),
                is_promoted: promoted == 1,
                is_running: false,
            };

            if let Some(icon) = &item.icon_guid {
                let guid = icon.trim_start_matches("{").trim_end_matches("}");
                let identifier = NOTIFYICONIDENTIFIER {
                    cbSize: std::mem::size_of::<NOTIFYICONIDENTIFIER>() as u32,
                    guidItem: GUID::try_from(guid)?,
                    ..Default::default()
                };
                item.is_running = unsafe { Shell_NotifyIconGetRect(&identifier).is_ok() };
            } else if let Some(uid) = &item.icon_uid {
                let str_item_exe_path = item.executable_path.to_string_lossy().to_lowercase();
                item.is_running = windows.iter().any(|(hwnd, path)| {
                    path == &str_item_exe_path && {
                        let identifier = NOTIFYICONIDENTIFIER {
                            cbSize: std::mem::size_of::<NOTIFYICONIDENTIFIER>() as u32,
                            hWnd: HWND(*hwnd as _),
                            uID: *uid,
                            ..Default::default()
                        };
                        unsafe { Shell_NotifyIconGetRect(&identifier).is_ok() }
                    }
                });
            }

            if item.is_running {
                registers.push(item);
                // send to tray overflow
                regkey.set_value("IsPromoted", &0u32)?;
            } else {
                // send to taskbar (outside tray overflow) items that are not running
                // this is done to filter items that can't be handled, as example the old nvidia control panel tray
                // these are unhandable cuz they are added from another user (SYSTEM)
                regkey.set_value("IsPromoted", &1u32)?;
            }
        }
        Ok(registers)
    }
}

pub fn get_sub_tree(
    element: &IUIAutomationElement,
    condition: &IUIAutomationCondition,
    scope: TreeScope,
) -> Result<Vec<IUIAutomationElement>> {
    let mut elements = Vec::new();
    unsafe {
        let element_array = element.FindAll(scope, condition)?;
        for index in 0..element_array.Length()? {
            let element = element_array.GetElement(index)?;
            elements.push(element);
        }
    }
    Ok(elements)
}

pub fn get_tray_overflow_handle() -> Option<HWND> {
    unsafe {
        FindWindowA(
            if is_windows_10() {
                pcstr!("NotifyIconOverflowWindow")
            } else {
                pcstr!("TopLevelWindowForOverflowXamlIsland")
            },
            None,
        )
        .ok()
    }
}

pub fn get_tray_overflow_content_handle() -> Option<HWND> {
    let tray_overflow = get_tray_overflow_handle()?;
    unsafe {
        FindWindowExA(
            Some(tray_overflow),
            None,
            if is_windows_10() {
                pcstr!("ToolbarWindow32")
            } else {
                pcstr!("Windows.UI.Composition.DesktopWindowContentBridge")
            },
            None,
        )
        .ok()
    }
}

static TRAY_CREATION_ATTEMPTS: AtomicU32 = AtomicU32::new(0);
pub fn ensure_tray_overflow_creation() -> Result<()> {
    if !is_windows_11() || get_tray_overflow_content_handle().is_some() {
        return Ok(());
    }

    let attempts = TRAY_CREATION_ATTEMPTS.load(Ordering::Acquire) + 1;
    if attempts > 10 {
        return Err("Maximum tray creation attempts reached".into());
    }

    TRAY_CREATION_ATTEMPTS.store(attempts, Ordering::Release);
    if attempts >= 10 {
        log::warn!("Tray overflow not created, and maximum attemps reached");
    }

    TrayIconManager::enable_chevron()?;
    TrayIconManager::send_all_to_tray_overflow()?;

    Com::run_with_context(|| unsafe {
        let tray_hwnd = get_native_shell_hwnd()?;

        let tray_bar = AppBarData::from_handle(tray_hwnd);
        let tray_bar_state = tray_bar.state();
        // This function will fail if taskbar is hidden
        tray_bar.set_state(AppBarDataState::AlwaysOnTop);
        WindowsApi::show_window_async(tray_hwnd, SW_SHOW)?;

        let automation: IUIAutomation = Com::create_instance(&CUIAutomation)?;
        let condition = automation.CreateTrueCondition()?;
        let element: IUIAutomationElement = automation.ElementFromHandle(tray_hwnd)?;

        let element_array = element.FindAll(TreeScope_Subtree, &condition)?;
        for index in 0..element_array.Length().unwrap_or(0) {
            let element = element_array.GetElement(index)?;
            if element.CurrentAutomationId()? == "SystemTrayIcon"
                && element.CurrentClassName()? == "SystemTray.NormalButton"
            {
                let invoker = element
                    .GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)?;
                // open and close the tray to force the creation of the overflow list
                invoker.Invoke()?;
                sleep_millis(10);
                invoker.Invoke()?;
                break;
            }
        }

        tray_bar.set_state(tray_bar_state);
        Ok(())
    })?;

    if get_tray_overflow_content_handle().is_none() {
        return Err("Failed to create tray overflow".into());
    }
    Ok(())
}

pub struct TrayIconUIElement(IUIAutomationElement);

impl TrayIconUIElement {
    pub fn invoke(&self) -> Result<()> {
        unsafe {
            let invoker = self
                .0
                .GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)?;
            invoker.Invoke()?;
        }
        Ok(())
    }

    pub fn context_menu(&self) -> Result<()> {
        let element: IUIAutomationElement3 = self.0.cast()?;

        let mut cursor_pos = POINT::default();
        unsafe { GetCursorPos(&mut cursor_pos as *mut POINT)? };

        if let Some(hwnd) = get_tray_overflow_handle() {
            WindowsApi::show_window_async(hwnd, SW_SHOW)?;
            let rect = WindowsApi::get_outer_window_rect(hwnd)?;

            WindowsApi::move_window(
                hwnd,
                &RECT {
                    top: cursor_pos.y - (rect.bottom - rect.top),
                    left: cursor_pos.x - (rect.right - rect.left),
                    right: 0,
                    bottom: 0,
                },
            )?;

            unsafe { element.ShowContextMenu()? };
            sleep_millis(500);
            WindowsApi::show_window_async(hwnd, SW_HIDE)?;
        }

        Ok(())
    }
}
