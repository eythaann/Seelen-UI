use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    UI::{
        Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
            IUIAutomationElement3, IUIAutomationInvokePattern, TreeScope, TreeScope_Children,
            TreeScope_Subtree, UIA_InvokePatternId,
        },
        Shell::{Shell_NotifyIconGetRect, NOTIFYICONIDENTIFIER},
        WindowsAndMessaging::{FindWindowA, FindWindowExA, GetCursorPos, SW_HIDE, SW_SHOW},
    },
};
use windows_core::{Interface, GUID};
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS},
    RegKey,
};

use crate::{
    error_handler::Result,
    pcstr,
    utils::{is_windows_10, is_windows_11, resolve_guid_path, sleep_millis},
    windows_api::{AppBarData, AppBarDataState, Com, WindowEnumerator, WindowsApi},
};

use super::domain::{RegistryNotifyIcon, TrayIcon};

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
        if is_windows_10() {
            FindWindowA(pcstr!("NotifyIconOverFlowWindow"), None).ok()
        } else {
            FindWindowA(pcstr!("TopLevelWindowForOverflowXamlIsland"), None).ok()
        }
    }
}

pub fn get_tray_overflow_content_handle() -> Option<HWND> {
    let tray_overflow = get_tray_overflow_handle()?;
    unsafe {
        if is_windows_10() {
            FindWindowExA(
                tray_overflow,
                HWND::default(),
                pcstr!("ToolbarWindow32"),
                None,
            )
            .ok()
        } else {
            FindWindowExA(
                tray_overflow,
                HWND::default(),
                pcstr!("Windows.UI.Composition.DesktopWindowContentBridge"),
                None,
            )
            .ok()
        }
    }
}

pub fn ensure_tray_overflow_creation() -> Result<()> {
    if !is_windows_11() || get_tray_overflow_content_handle().is_some() {
        return Ok(());
    }

    TrayIconManager::enable_chevron()?;

    Com::run_with_context(|| unsafe {
        let tray_hwnd = FindWindowA(pcstr!("Shell_TrayWnd"), None)?;

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

pub fn get_tray_icons() -> Result<Vec<TrayIcon>> {
    ensure_tray_overflow_creation()?;
    let tray_from_registry = TrayIconManager::enum_from_registry()?;

    Com::run_with_context(|| unsafe {
        let mut tray_elements = Vec::new();

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
                    ui_automation: element,
                });
                idx += 1;
            }
        }
        Ok(tray_elements)
    })
}

impl TrayIcon {
    pub fn invoke(&self) -> Result<()> {
        unsafe {
            let invoker = self
                .ui_automation
                .GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)?;
            invoker.Invoke()?;
        }
        Ok(())
    }

    pub fn context_menu(&self) -> Result<()> {
        let element: IUIAutomationElement3 = self.ui_automation.cast()?;

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

pub struct TrayIconManager {}
impl TrayIconManager {
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
        WindowEnumerator::new().for_each_v2(|w| {
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
                    guidItem: GUID::from(guid),
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
