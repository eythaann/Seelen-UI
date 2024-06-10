use itertools::Itertools;
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    },
    UI::{
        Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationInvokePattern,
            TreeScope_Descendants, UIA_InvokePatternId,
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
    utils::{is_windows_11_22h2, resolve_guid_path},
    windows_api::WindowsApi,
};

use super::domain::{RegistryNotifyIcon, TrayIcon, TrayIconInfo};

pub fn get_tray_handle() -> HWND {
    unsafe {
        // https://learn.microsoft.com/en-us/answers/questions/1483214/win11-22h2-(10-0-22621)-cant-support-tb-buttoncount
        if is_windows_11_22h2() {
            let tray_overflow_hwnd = FindWindowA(None, pcstr!("System tray overflow window."));
            let tray_overflow_list_hwnd = FindWindowExA(
                tray_overflow_hwnd,
                HWND(0),
                None,
                pcstr!("DesktopWindowXamlSource"),
            );

            return tray_overflow_list_hwnd;
        }

        // Todo test on windows 10
        let tray_hwnd = FindWindowA(pcstr!("Shell_TrayWnd"), None);
        let tray_notify_hwnd = FindWindowExA(tray_hwnd, HWND(0), pcstr!("TrayNotifyWnd"), None);
        let sys_pager_hwnd = FindWindowExA(tray_notify_hwnd, HWND(0), pcstr!("SysPager"), None);
        let toolbar_hwnd = FindWindowExA(sys_pager_hwnd, HWND(0), pcstr!("ToolbarWindow32"), None);

        toolbar_hwnd
    }
}

/*
FOR TASKBAR ICONS:
let rebar_hwnd = FindWindowExA(tray_hwnd, HWND(0), s!("ReBarWindow32"), None);
let task_hwnd = FindWindowExA(rebar_hwnd, HWND(0), s!("MSTaskSwWClass"), None);
let task_list_hwnd = FindWindowExA(task_hwnd, HWND(0), s!("MSTaskListWClass"), None); */
pub fn get_tray_icons() -> Result<Vec<TrayIcon>> {
    let mut tray_elements = Vec::new();
    let tray_from_registry = TrayIcon::enum_from_registry()?;

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;

        let automation: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)?;
        let condition = automation.CreateTrueCondition()?;

        let element: IUIAutomationElement = automation.ElementFromHandle(get_tray_handle())?;

        let element_array = element.FindAll(TreeScope_Descendants, &condition);
        if let Ok(element_array) = element_array {
            for index in 0..element_array.Length().unwrap_or(0) {
                if let Ok(element) = element_array.GetElement(index) {
                    if element.CurrentAutomationId()? == "NotifyItemIcon" {
                        let name = element.CurrentName()?.to_string();
                        let registry = tray_from_registry
                            .iter()
                            .find(|t| t.initial_tooltip == name);

                        if let Some(registry) = registry {
                            tray_elements.push(TrayIcon {
                                ui_automation: element,
                                registry: registry.clone(),
                            });
                        }
                    }
                }
            }
        }

        CoUninitialize();
    }

    Ok(tray_elements)
}

impl TrayIcon {
    pub fn info(&self) -> TrayIconInfo {
        TrayIconInfo {
            icon: match self.icon() {
                Ok(icon) => icon,
                Err(e) => {
                    log::error!("Failed to get tray icon: {}", e);
                    String::from("")
                }
            },
            label: self.name().unwrap_or("Unknown".to_string()),
        }
    }

    pub fn name(&self) -> Result<String> {
        Ok(unsafe { self.ui_automation.CurrentName() }?.to_string())
    }

    pub fn icon(&self) -> Result<String> {
        let path = &self.registry.executable_path;
        let icon = extract_and_save_icon(&get_app_handle(), path)?;
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
            let key = settings.open_subkey(id.to_string())?;

            if key.get_raw_value("IconGuid").is_ok() {
                // TODO: Handle Tray Icons like USB devices, Security tray, etc
                continue;
            };

            // executable path should always exist in registry
            let path: String = key.get_value("ExecutablePath")?;
            let executable_path = resolve_guid_path(path)?.to_string_lossy().to_string();

            let is_executing = processes_string.contains(&executable_path.to_lowercase());
            if is_executing {
                let promoted: u32 = key.get_value("IsPromoted").unwrap_or_default();

                registers.push(RegistryNotifyIcon {
                    executable_path,
                    initial_tooltip: key.get_value("InitialTooltip").unwrap_or_default(),
                    is_promoted: promoted != 0,
                })
            }
        }

        Ok(registers)
    }
}
