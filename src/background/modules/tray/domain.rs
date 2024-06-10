use serde::Serialize;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

#[derive(Debug, Clone, Serialize)]
pub struct TrayIconInfo {
    pub icon: String,
    pub label: String,
}

pub struct TrayIcon {
    pub ui_automation: IUIAutomationElement,
    pub registry: RegistryNotifyIcon,
}


#[derive(Debug, Clone)]
pub struct RegistryNotifyIcon {
    pub executable_path: String,
    pub initial_tooltip: String,
    pub is_promoted: bool,
}
