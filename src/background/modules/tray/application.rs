use itertools::Itertools;
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    System::Registry,
    UI::{
        Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
            IUIAutomationInvokePattern, TreeScope, TreeScope_Descendants, TreeScope_Subtree,
            UIA_InvokePatternId,
        },
        WindowsAndMessaging::{FindWindowA, FindWindowExA, GetCursorPos, SW_SHOW},
    },
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::{
    error_handler::Result,
    modules::input::Keyboard,
    pcstr,
    seelen::get_app_handle,
    seelen_weg::icon_extractor::extract_and_save_icon,
    utils::{is_windows_10, is_windows_11, resolve_guid_path, sleep_millis},
    windows_api::{AppBarData, AppBarDataState, Com, WindowsApi},
};

use super::domain::{RegistryNotifyIcon, TrayIcon, TrayIconInfo};

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

// force_tray_overflow_creation should be called before get_tray_handle
// https://learn.microsoft.com/en-us/answers/questions/1483214/win11-22h2-(10-0-22621)-cant-support-tb-buttoncount
pub fn get_tray_overflow_handle() -> HWND {
    unsafe {
        if is_windows_10() {
            let tray_overflow = FindWindowA(pcstr!("NotifyIconOverFlowWindow"), None);
            FindWindowExA(tray_overflow, HWND(0), pcstr!("ToolbarWindow32"), None)
        } else {
            let tray_overflow = FindWindowA(pcstr!("TopLevelWindowForOverflowXamlIsland"), None);
            FindWindowExA(
                tray_overflow,
                HWND(0),
                None,
                pcstr!("DesktopWindowXamlSource"),
            )
        }
    }
}

pub fn try_force_tray_overflow_creation() -> Result<()> {
    if !is_windows_11() {
        return Ok(());
    }

    Com::run_with_context(|| unsafe {
        let tray_overflow_hwnd = FindWindowA(pcstr!("TopLevelWindowForOverflowXamlIsland"), None);
        if tray_overflow_hwnd.0 != 0 {
            return Ok(());
        }

        let tray_hwnd = FindWindowA(pcstr!("Shell_TrayWnd"), None);

        let tray_bar = AppBarData::from_handle(tray_hwnd);
        let tray_bar_state = tray_bar.state();
        // This function will fail if taskbar is hidden
        tray_bar.set_state(AppBarDataState::AlwaysOnTop);

        let automation: IUIAutomation = Com::create_instance(&CUIAutomation)?;
        let condition = automation.CreateTrueCondition()?;
        let element: IUIAutomationElement = automation.ElementFromHandle(tray_hwnd)?;

        let element_array = element.FindAll(TreeScope_Subtree, &condition)?;
        for index in 0..element_array.Length().unwrap_or(0) {
            let element = element_array.GetElement(index)?;
            if element.CurrentName()?.to_string() == "Show Hidden Icons"
                && element.CurrentAutomationId()?.to_string() == "SystemTrayIcon"
            {
                let invoker = element
                    .GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)?;
                // open and close the tray to force the creation of the overflow list
                invoker.Invoke()?;
                sleep_millis(10);
                invoker.Invoke()?;

                tray_bar.set_state(tray_bar_state);
                return Ok(());
            }
        }

        tray_bar.set_state(tray_bar_state);
        Err("Failed to force tray overflow creation".into())
    })
}

/*
FOR TASKBAR ICONS:
let rebar_hwnd = FindWindowExA(tray_hwnd, HWND(0), s!("ReBarWindow32"), None);
let task_hwnd = FindWindowExA(rebar_hwnd, HWND(0), s!("MSTaskSwWClass"), None);
let task_list_hwnd = FindWindowExA(task_hwnd, HWND(0), s!("MSTaskListWClass"), None); */

pub fn get_tray_icons() -> Result<Vec<TrayIcon>> {
    let tray_from_registry = TrayIcon::enum_from_registry()?;

    Com::run_with_context(|| unsafe {
        let mut tray_elements = Vec::new();

        let automation: IUIAutomation = Com::create_instance(&CUIAutomation)?;
        let condition = automation.CreateTrueCondition()?;

        let mut children = Vec::new();

        let tray_overflow = get_tray_overflow_handle();
        if tray_overflow.0 != 0 {
            let element: IUIAutomationElement = automation.ElementFromHandle(tray_overflow)?;
            children.extend(get_sub_tree(&element, &condition, TreeScope_Descendants)?);
        }

        let is_win10 = is_windows_10();
        for element in children {
            if is_win10 || element.CurrentAutomationId()? == "NotifyItemIcon" {
                let name = element.CurrentName()?.to_string();

                let registry = tray_from_registry.iter().find(|t| {
                    let trimmed = name.trim();
                    t.initial_tooltip == trimmed
                        || t.executable_path
                            .to_lowercase()
                            .contains(&trimmed.to_lowercase())
                });

                tray_elements.push(TrayIcon {
                    ui_automation: element,
                    registry: registry.as_deref().clone().cloned(),
                });
            }
        }

        Ok(tray_elements)
    })
}

impl TrayIcon {
    pub fn info(&self) -> TrayIconInfo {
        TrayIconInfo {
            icon: self.icon().ok(),
            label: self.name().unwrap_or("Unknown".to_string()),
        }
    }

    pub fn name(&self) -> Result<String> {
        Ok(unsafe { self.ui_automation.CurrentName() }?.to_string())
    }

    pub fn icon(&self) -> Result<String> {
        if self.registry.is_none() {
            return Err("Registry icon not found".into());
        }

        let path = self.registry.as_ref().unwrap().executable_path.clone();
        let icon = extract_and_save_icon(&get_app_handle(), &path)?;
        Ok(icon
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string())
    }

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
        let element = &self.ui_automation;
        let mut cursor_pos = POINT::default();
        unsafe { GetCursorPos(&mut cursor_pos as *mut POINT)? };

        let hwnd = unsafe { FindWindowA(None, pcstr!("System tray overflow window.")) };

        WindowsApi::show_window(hwnd, SW_SHOW)?;
        WindowsApi::move_window(
            hwnd,
            &RECT {
                top: cursor_pos.y,
                left: cursor_pos.x,
                right: 0,
                bottom: 0,
            },
        )?;

        unsafe { element.SetFocus()? };
        Keyboard::new().send_keys("{apps}")?;

        Ok(())
    }

    pub fn enum_from_registry() -> Result<Vec<RegistryNotifyIcon>> {
        let hkcr = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcr.open_subkey("Control Panel\\NotifyIconSettings")?;
        let list = settings.get_raw_value("UIOrderList")?;

        // the order in this list is the order in which the icons will be displayed on the Win Taskbar and Win Tray Overflow
        let ids = list
            .bytes
            .chunks_exact(8)
            .map(|chunk| {
                u64::from_le_bytes(chunk.try_into().expect("Registry value is not 8 bytes"))
            })
            .collect_vec();

        let mut registers = Vec::new();

        let sys = sysinfo::System::new_all();
        let mut processes_string = Vec::new();
        sys.processes().values().for_each(|p| {
            if let Some(exe) = p.exe() {
                processes_string.push(exe.to_string_lossy().to_string().to_lowercase());
            }
        });

        for id in ids {
            let key =
                settings.open_subkey_with_flags(id.to_string(), Registry::KEY_ALL_ACCESS.0)?;

            let promoted: u32 = key.get_value("IsPromoted").unwrap_or_default();
            if promoted == 1 && WindowsApi::is_elevated()? {
                // avoid show tray icons directly on taskbar
                // all icons should be in the tray overflow window
                key.set_value("IsPromoted", &0u32)?;
            }

            if key.get_raw_value("IconGuid").is_ok() {
                // TODO: Handle Tray Icons like USB devices, Security tray, etc
                continue;
            };

            // executable path should always exist in registry
            let path: String = key.get_value("ExecutablePath")?;
            let executable_path = resolve_guid_path(path)?.to_string_lossy().to_string();

            let is_executing = processes_string.contains(&executable_path.to_lowercase());
            if is_executing {
                registers.push(RegistryNotifyIcon {
                    executable_path,
                    initial_tooltip: key.get_value("InitialTooltip").unwrap_or_default(),
                })
            }
        }

        Ok(registers)
    }
}
