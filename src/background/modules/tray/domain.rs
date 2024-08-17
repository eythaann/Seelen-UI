use serde::Serialize;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

#[derive(Debug, Clone, Serialize)]
pub struct TrayIconInfo {
    pub icon: Option<String>,
    pub label: String,
}

pub struct TrayIcon {
    pub ui_automation: IUIAutomationElement,
    pub registry: Option<RegistryNotifyIcon>,
}

#[derive(Debug, Clone)]
pub struct RegistryNotifyIcon {
    pub executable_path: String,
    pub initial_tooltip: String,
}
