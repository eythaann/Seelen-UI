use std::collections::HashMap;

use crate::resource::{PluginId, WidgetId};
use crate::state::*;
use crate::system_state::*;

macro_rules! slu_events_declaration {
    ($(
        $(@queued($($is_queued:tt)?))?
        $name:ident$(($payload:ty))? as $value:literal,
    )*) => {
        pub struct SeelenEvent;

        #[allow(non_upper_case_globals)]
        impl SeelenEvent {
            $(
                pub const $name: &'static str = $value;
            )*

            /// Events that must be delivered one by one. Any other event is a full-state
            /// snapshot, so transports may coalesce it and deliver only its latest value.
            const QUEUED: &'static [&'static str] = &[
                $($(Self::$name, $($is_queued)?)?)*
            ];

            /// Whether the given name is a declared event.
            pub fn exists(event: &str) -> bool {
                [$(Self::$name),*].contains(&event)
            }

            /// Whether the event can't be coalesced (it carries an action or a delta, not a snapshot).
            pub fn is_queued(event: &str) -> bool {
                Self::QUEUED.contains(&event)
            }

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

        #[derive(Serialize)]
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
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

    SystemMonitorsChanged(Vec<PhysicalMonitor>) as "system::monitors-changed",
    SystemLanguagesChanged(Vec<SystemLanguage>) as "system::languages-changed",
    SystemImeStateChanged(ImeState) as "system::ime-state-changed",

    UserChanged(User) as "user-changed",
    @queued()
    UserFolderChanged(FolderChangedArgs) as "user::known-folder-changed",
    UserAppWindowsChanged(Vec<UserAppWindow>) as "user::windows-changed",
    UserAppWindowsPreviewsChanged(HashMap<isize, UserAppWindowPreview>) as "user::windows-previews-changed",
    UserAppWindowsColorsChanged(HashMap<isize, UserAppWindowColors>) as "user::windows-colors-changed",

    MediaSessions(Vec<MediaPlayer>) as "media-sessions",
    MediaDevices([Vec<MediaDevice>; 2]) as "media::devices",
    MediaInputs(Vec<MediaDevice>) as "media-inputs",
    MediaOutputs(Vec<MediaDevice>) as "media-outputs",
    MediaWaveform(AudioWaveform) as "media::waveform",

    NetworkDefaultLocalIp(String) as "network-default-local-ip",
    NetworkAdapters(Vec<NetworkAdapter>) as "network-adapters",
    NetworkInternetConnection(bool) as "network-internet-connection",
    NetworkWlanScanned(Vec<WlanBssEntry>) as "wlan-scanned",
    NetworkHotspotChanged(Option<Hotspot>) as "network-hotspot-changed",

    PowerStatus(PowerStatus) as "power-status",
    PowerMode(PowerMode) as "power-mode",
    BatteriesStatus(Vec<Battery>) as "batteries-status",

    ColorsChanged(UIColors) as "colors-changed",
    DarkModeChanged(bool) as "dark-mode-changed",
    SystemFontsChanged(Vec<SeelenFont>) as "system::fonts-changed",

    WMSetReservation(Option<twm::TwmReservation>) as "wm::set-reservation",
    WMForceRetiling as "wm::force-retiling",
    WMTreeChanged(TwmGlobalRuntimeTree) as "wm::tree-changed",

    StateSettingsChanged(Settings) as "settings-changed",
    StateThemesChanged(Vec<Theme>) as "themes",
    StateIconPacksChanged(Vec<IconPack>) as "icon-packs",
    StatePluginsChanged(Vec<Plugin>) as "plugins-changed",
    StateWidgetsChanged(Vec<Widget>) as "widgets-changed",
    WidgetPermissionsChanged(HashMap<WidgetId, HashMap<WidgetPerm, WidgetPermState>>) as "widget::permissions-changed",
    StateWallpapersChanged(Vec<Wallpaper>) as "UserResources::wallpapers-changed",

    // system tray
    SystemTrayChanged(Vec<SysTrayIcon>) as "system-tray::changed",

    StatePerformanceModeChanged(PerformanceMode) as "state::performance-mode-changed",

    @queued()
    WidgetTriggered(WidgetTriggerPayload) as "widget::triggered",

    // Radios
    RadiosChanged(Vec<RadioDevice>) as "radio::changed",

    // System Info
    SystemDisksChanged(Vec<Disk>) as "system::disks-changed",
    SystemNetworkChanged(Vec<NetworkStatistics>) as "system::network-changed",
    SystemMemoryChanged(Memory) as "system::memory-changed",
    SystemCoresChanged(Vec<Core>) as "system::cores-changed",

    BluetoothDevicesChanged(Vec<BluetoothDevice>) as "bluetooth-devices-changed",

    // Start Menu
    StartMenuItemsChanged(Vec<StartMenuItem>) as "start-menu::items-changed",

    // SeelenWeg
    @queued()
    WegAddItem(WegItemData) as "weg::add-item",
    WegItemsChanged(WegItems) as "weg::items-changed",

    // Trash Bin
    TrashBinChanged(TrashBinInfo) as "trash-bin::changed",

    // Seelen Session
    SeelenSessionChanged(Option<SeelenSession>) as "session::changed",

    // Cloud Backup
    SeelenBackupStatusChanged(BackupStatus) as "backup::status-changed",

    // Clipboard
    ClipboardDataChanged(ClipboardData) as "clipboard::data-changed",

    // Notifications
    FocusAssistChanged(bool) as "focus-assist::changed",
    NotificationsModeChanged(NotificationsMode) as "notifications::mode-changed",
    Notifications(Vec<AppNotification>) as "notifications",

    // Plugins
    @queued()
    PluginEnabled(PluginId) as "plugin::enabled",

    // Shortcuts
    ShortcutsPaused(bool) as "shortcuts::paused",

    // Widget debug info
    WidgetDebugInfoChanged(Vec<WidgetDebugInfo>) as "widget::debug-info-changed",
}

/// Credentials of a webview to connect to the global events websocket.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct SelfEventsToken {
    /// Identifies the webview, sent as the `token` query param.
    pub token: String,
    /// Websocket url, e.g. `ws://127.0.0.1:37074/events`.
    pub endpoint: String,
}
