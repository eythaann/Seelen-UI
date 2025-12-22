#[cfg(test)]
use crate::{
    rect::Rect, resource::*, state::by_monitor::MonitorConfiguration,
    state::by_wallpaper::WallpaperInstanceSettings, state::*, system_state::*,
};
#[cfg(test)]
use std::{collections::HashMap, path::PathBuf};

macro_rules! slu_commands_declaration {
    ($($key:ident = $fn_name:ident($($args:tt)*) $(-> $return_type:ty)?,)*) => {
        #[cfg(test)]
        pub struct SeelenCommand;

        #[cfg(test)]
        impl SeelenCommand {
            #[cfg(feature = "gen-binds")]
            pub(crate) fn generate_ts_file(path: &str) {
                let mut content: Vec<String> = std::vec::Vec::new();

                content.push("// This file was generated via rust macros. Don't modify manually.".to_owned());
                content.push("export enum SeelenCommand {".to_owned());
                $(
                    content.push(format!("  {} = '{}',", stringify!($key), stringify!($fn_name)));
                )*
                content.push("}\n".to_owned());

                std::fs::write(path, content.join("\n")).unwrap();
            }
        }

        paste::paste! {
            $(
                $crate::__switch! {
                    if { $($args)* }
                    do {
                        #[cfg(test)]
                        #[derive(Deserialize, TS)]
                        #[serde(rename_all = "camelCase")]
                        #[allow(dead_code)]
                        struct [<SeelenCommand $key Args>] {
                            $($args)*
                        }
                    }
                    else {}
                }
            )*

            /// Internal used as mapping of commands to their arguments
            #[cfg(test)]
            #[allow(non_camel_case_types, dead_code)]
            #[derive(Deserialize, TS)]
            #[cfg_attr(feature = "gen-binds", ts(export))]
            enum SeelenCommandArgument {
                $(
                    #[allow(non_snake_case)]
                    $fn_name(Box<$crate::__switch! {
                        if { $($args)* }
                        do { [<SeelenCommand $key Args>] }
                        else { () }
                    }>),
                )*
            }
        }

        /// Internal used as mapping of commands to their return types
        #[derive(Serialize, TS)]
        #[cfg_attr(feature = "gen-binds", ts(export))]
        #[allow(non_camel_case_types, dead_code)]
        #[cfg(test)]
        enum SeelenCommandReturn {
            $(
                $fn_name(Box<$crate::__switch! {
                    if { $($return_type)? }
                    do { $($return_type)? }
                    else { () }
                }>),
            )*
        }

        #[macro_export]
        macro_rules! command_handler_list {
            () => {
                tauri::generate_handler![
                    $(
                        $fn_name,
                    )*
                ]
            };
        }

        pub use command_handler_list;
    };
}

slu_commands_declaration! {
    // virtual desktops
    StateGetVirtualDesktops = get_virtual_desktops() -> VirtualDesktops,
    SwitchWorkspace = switch_workspace(monitor_id: MonitorId, idx: usize),

    // General
    Run = run(program: PathBuf, args: Option<RelaunchArguments>, working_dir: Option<PathBuf>),
    RunAsAdmin = run_as_admin(program: PathBuf, args: Option<RelaunchArguments>),

    GetFocusedApp = get_focused_app() -> FocusedApp,
    GetMousePosition = get_mouse_position() -> [i32; 2],

    IsDevMode = is_dev_mode() -> bool,
    IsAppxPackage = is_appx_package() -> bool,
    OpenFile = open_file(path: PathBuf),
    SelectFileOnExplorer = select_file_on_explorer(path: PathBuf),
    GetUserEnvs = get_user_envs() -> HashMap<String, String>,
    ShowAppSettings = show_app_settings(),
    SendKeys = send_keys(keys: String),
    GetIcon = get_icon(
        #[ts(optional = nullable)]
        path: Option<PathBuf>,
        #[ts(optional = nullable)]
        umid: Option<String>
    ),
    SimulateFullscreen = simulate_fullscreen(),
    ShowDesktop = show_desktop(),

    RequestToUserInputShortcut = request_to_user_input_shortcut(callback_event: String),

    CheckForUpdates = check_for_updates() -> bool,
    // Restart the app after install the update so it returns a promise resolved with `never`
    InstallLastAvailableUpdate = install_last_available_update(),

    // System
    SystemGetForegroundWindowColor = get_foreground_window_color() -> Color,
    SystemGetMonitors = get_connected_monitors() -> Vec<PhysicalMonitor>,
    SystemGetColors = get_system_colors() -> UIColors,
    SystemGetLanguages = get_system_languages() -> Vec<SystemLanguage>,
    SystemSetKeyboardLayout = set_system_keyboard_layout(id: String, handle: String),

    // Seelen Settings
    StateGetDefaultSettings = state_get_default_settings() -> Settings,
    StateGetDefaultMonitorSettings = state_get_default_monitor_settings() -> MonitorConfiguration,
    StateGetDefaultWallpaperSettings = state_get_default_wallpaper_settings() -> WallpaperInstanceSettings,

    SetAutoStart = set_auto_start(enabled: bool),
    GetAutoStartStatus = get_auto_start_status() -> bool,
    RemoveResource = remove_resource(id: ResourceId, kind: ResourceKind),

    StateGetThemes = state_get_themes() -> Vec<Theme>,
    StateGetWegItems = state_get_weg_items(monitor_id: Option<MonitorId>) -> WegItems,
    StateWriteWegItems = state_write_weg_items(items: WegItems),
    StateGetToolbarItems = state_get_toolbar_items() -> Placeholder,
    StateGetSettings = state_get_settings(path: Option<PathBuf>) -> Settings,
    StateWriteSettings = state_write_settings(settings: Settings),
    StateGetSpecificAppsConfigurations = state_get_specific_apps_configurations() -> Vec<AppConfig> ,
    StateGetHistory = state_get_history() -> LauncherHistory,
    StateGetPlugins = state_get_plugins() -> Vec<Plugin>,
    StateGetWidgets = state_get_widgets() -> Vec<Widget>,
    StateGetIconPacks = state_get_icon_packs() -> Vec<IconPack>,
    StateGetWallpapers = state_get_wallpapers() -> Vec<Wallpaper>,
    StateSetCustomIconPack = state_add_icon_to_custom_icon_pack(icon: IconPackEntry),
    StateGetProfiles = state_get_profiles() -> Vec<Profile>,
    StateDeleteCachedIcons = state_delete_cached_icons(),
    StateRequestWallpaperAddition = state_request_wallpaper_addition(),
    StateGetPerformanceMode = state_get_performance_mode() -> PerformanceMode,

    // Widgets
    TriggerWidget = trigger_widget(payload: WidgetTriggerPayload),
    SetCurrentWidgetStatus = set_current_widget_status(status: WidgetStatus),
    GetSelfWindowId = get_self_window_handle() -> isize,

    // Shell
    GetNativeShellWallpaper = get_native_shell_wallpaper() -> PathBuf,
    SetNativeShellWallpaper = set_native_shell_wallpaper(path: PathBuf),

    // User
    GetUser = get_user() -> User,
    GetUserFolderContent = get_user_folder_content(folder_type: FolderType) -> Vec<File>,
    SetUserFolderLimit = set_user_folder_limit(folder_type: FolderType, amount: usize),
    GerUserApplications = get_user_applications() -> Vec<UserApplication>,
    GetUserAppWindows = get_user_app_windows() -> Vec<UserAppWindow>,
    GetUserAppWindowsPreviews = get_user_app_windows_previews() -> HashMap<isize, UserAppWindowPreview>,

    // Media
    GetMediaDevices = get_media_devices() -> [Vec<MediaDevice>; 2],
    GetMediaSessions = get_media_sessions() -> Vec<MediaPlayer>,
    MediaPrev = media_prev(id: String),
    MediaTogglePlayPause = media_toggle_play_pause(id: String),
    MediaNext = media_next(id: String),
    SetVolumeLevel = set_volume_level(device_id: String, session_id: Option<String>, level: f32),
    MediaToggleMute = media_toggle_mute(device_id: String, session_id: Option<String>),
    MediaSetDefaultDevice = media_set_default_device(id: String, role: String),

    // Brightness
    GetMainMonitorBrightness = get_main_monitor_brightness() -> Option<Brightness>,
    SetMainMonitorBrightness = set_main_monitor_brightness(brightness: u8),

    // Power
    GetPowerStatus = get_power_status() -> PowerStatus,
    GetPowerMode = get_power_mode() -> PowerMode,
    GetBatteries = get_batteries() -> Vec<Battery>,
    LogOut = log_out(),
    Suspend = suspend(),
    Hibernate = hibernate(),
    Restart = restart(),
    Shutdown = shutdown(),
    Lock = lock(),

    // SeelenWeg
    WegCloseApp = weg_close_app(hwnd: isize),
    WegKillApp = weg_kill_app(hwnd: isize),
    WegToggleWindowState = weg_toggle_window_state(hwnd: isize, was_focused: bool),
    WegPinItem = weg_pin_item(path: PathBuf),

    // Windows Manager
    WmGetRenderTree = wm_get_render_tree() -> WmRenderTree,
    SetAppWindowsPositions = set_app_windows_positions(positions: HashMap<isize, Rect>),
    RequestFocus = request_focus(hwnd: isize),

    // App Launcher
    LauncherGetApps = launcher_get_apps() -> Vec<StartMenuItem>,

    // Slu Popups
    CreatePopup = create_popup(config: SluPopupConfig) -> uuid::Uuid,
    UpdatePopup = update_popup(instance_id: uuid::Uuid, config: SluPopupConfig),
    ClosePopup = close_popup(instance_id: uuid::Uuid),
    GetPopupConfig = get_popup_config(instance_id: uuid::Uuid) -> SluPopupConfig,

    // Network
    WlanGetProfiles = wlan_get_profiles() -> Vec<WlanProfile>,
    WlanStartScanning = wlan_start_scanning(),
    WlanStopScanning = wlan_stop_scanning(),
    WlanConnect = wlan_connect(ssid: String, password: Option<String>, hidden: bool) -> bool,
    WlanDisconnect = wlan_disconnect(),

    // system tray
    GetSystemTrayIcons = get_system_tray_icons() -> Vec<SysTrayIcon>,
    SendSystemTrayIconAction = send_system_tray_icon_action(id: SysTrayIconId, action: SystrayIconAction),

    // Notifications
    GetNotifications = get_notifications() -> Vec<AppNotification>,
    NotificationsClose = notifications_close(id: u32),
    NotificationsCloseAll = notifications_close_all(),
    ActivateNotification = activate_notification(
        umid: String,
        args: String,
        input_data: HashMap<String, String>,
    ),

    // Radios
    GetRadios = get_radios() -> Vec<RadioDevice>,
    SetRadioState = set_radios_state(kind: RadioDeviceKind, enabled: bool),

    // Bluetooth
    GetBluetoothDevices = get_bluetooth_devices() -> Vec<BluetoothDevice>,
    StartBluetoothScanning = start_bluetooth_scanning(),
    StopBluetoothScanning = stop_bluetooth_scanning(),
    RequestPairBluetoothDevice = request_pair_bluetooth_device(id: String) -> DevicePairingNeededAction,
    ConfirmBluetoothDevicePairing = confirm_bluetooth_device_pairing(id: String, answer: DevicePairingAnswer),
    DisconnectBluetoothDevice = disconnect_bluetooth_device(id: String),
    ForgetBluetoothDevice = forget_bluetooth_device(id: String),
}
