use crate::state::*;
use crate::system_state::*;

macro_rules! slu_events_declaration {
    ($($name:ident$(($payload:ty))? as $value:literal,)*) => {
        pub struct SeelenEvent;

        #[allow(non_upper_case_globals)]
        impl SeelenEvent {
            $(
                pub const $name: &'static str = $value;
            )*

            #[allow(dead_code)]
            pub(crate) fn generate_ts_file(path: &str) {
                let content: Vec<String> = vec![
                    "// This file was generated via rust macros. Don't modify manually.".to_owned(),
                    "export enum SeelenEvent {".to_owned(),
                    $(
                        format!("  {} = '{}',", stringify!($name), Self::$name),
                    )*
                    "}\n".to_owned(),
                ];
                std::fs::write(path, content.join("\n")).unwrap();
            }
        }

        #[derive(Serialize, TS)]
        #[cfg_attr(feature = "gen-binds", ts(export))]
        pub enum SeelenEventPayload {
            $(
                #[serde(rename = $value)]
                $name($crate::__switch! {
                    if { $($payload)? }
                    do { Box<$($payload)?> }
                    else { () }
                }),
            )*
        }
    };
}

slu_events_declaration! {
    VirtualDesktopsChanged(VirtualDesktops) as "virtual-desktops::changed",

    GlobalFocusChanged(FocusedApp) as "global-focus-changed",
    GlobalMouseMove([i32; 2]) as "global-mouse-move",

    HandleLayeredHitboxes(bool) as "handle-layered",

    SystemMonitorsChanged(Vec<PhysicalMonitor>) as "system::monitors-changed",
    SystemLanguagesChanged(Vec<SystemLanguage>) as "system::languages-changed",

    UserChanged(User) as "user-changed",
    UserFolderChanged(FolderChangedArgs) as "user-folder-changed",
    UserApplicationsChanged(Vec<UserApplication>) as "user::applications-changed",
    UserAppWindowsChanged(Vec<UserAppWindow>) as "user::windows-changed",

    BluetoothDevicesChanged(Vec<BluetoothDevice>) as "bluetooth-devices-changed",
    BluetoothDiscoveredDevicesChanged(Vec<BluetoothDevice>) as "bluetooth-discovered-devices-changed",
    BluetoothPairShowPin(BluetoothDevicePairShowPinRequest) as "bluetooth-pair-show-pin",
    BluetoothPairRequestPin as "bluetooth-pair-request-pin",

    MediaSessions(Vec<MediaPlayer>) as "media-sessions",
    MediaInputs(Vec<MediaDevice>) as "media-inputs",
    MediaOutputs(Vec<MediaDevice>) as "media-outputs",

    NetworkDefaultLocalIp(String) as "network-default-local-ip",
    NetworkAdapters(Vec<NetworkAdapter>) as "network-adapters",
    NetworkInternetConnection(bool) as "network-internet-connection",
    NetworkWlanScanned(Vec<WlanBssEntry>) as "wlan-scanned",

    Notifications(Vec<AppNotification>) as "notifications",

    PowerStatus(PowerStatus) as "power-status",
    PowerMode(PowerMode) as "power-mode",
    BatteriesStatus(Vec<Battery>) as "batteries-status",

    ColorsChanged(UIColors) as "colors-changed",

    WegOverlaped(bool) as "set-auto-hide",

    WMSetReservation as "set-reservation",
    WMForceRetiling as "wm-force-retiling",
    WMSetLayout(WmNode) as "wm-set-layout",

    PopupContentChanged(SluPopupConfig) as "popup-content-changed",

    StateSettingsChanged(Settings) as "settings-changed",
    StateWegItemsChanged as "weg-items",
    StateToolbarItemsChanged(Placeholder) as "toolbar-items",
    StateThemesChanged(Vec<Theme>) as "themes",
    StateSettingsByAppChanged(Vec<AppConfig>) as "settings-by-app",
    StateHistoryChanged(LauncherHistory) as "history",
    StateIconPacksChanged(Vec<IconPack>) as "icon-packs",
    StatePluginsChanged(Vec<Plugin>) as "plugins-changed",
    StateWidgetsChanged(Vec<Widget>) as "widgets-changed",
    StateWallpapersChanged(Vec<Wallpaper>) as "UserResources::wallpapers-changed",
    StateProfilesChanged(Vec<Profile>) as "profiles-changed",

    // system tray
    SystemTrayChanged(Vec<SysTrayIcon>) as "system-tray::changed",

    StatePerformanceModeChanged(PerformanceMode) as "state::performance-mode-changed",

    WidgetTriggered(WidgetTriggerPayload) as "widget::triggered",
}
