#[cfg(feature = "gen-binds")]
use crate::{
    rect::Rect, resource::*, state::by_monitor::MonitorConfiguration,
    state::by_wallpaper::WallpaperInstanceSettings, state::context_menu::*,
    state::settings::shortcuts::SystemShortcutDeclaration, state::*, system_state::*,
    utils::TsVoid,
};

#[cfg(feature = "gen-binds")]
macro_rules! type_or_void {
    () => {
        TsVoid
    };
    ($type:ty) => {
        $type
    };
}

/// Selects between passing a command's result through as-is (when the command is
/// declared `@fallible`, so `Handlers::$fn_name` already returns `crate::Result<T>`)
/// or wrapping it in `Ok(..)` (when the command has no `@fallible` marker, so
/// `Handlers::$fn_name` returns a plain `T` that can never fail).
#[doc(hidden)]
#[macro_export]
macro_rules! __slu_command_result {
    (@fallible $e:expr) => {
        $e
    };
    ($e:expr) => {
        Ok($e)
    };
}

macro_rules! slu_commands_declaration {
    ($(
        $key:ident =
            $(@webview($($with_webview:tt)?))?
            $(@async($($with_async:tt)?))?
            $(@fallible($($is_fallible:tt)?))?
            $fn_name:ident(
            $(
                $(#[$attr:meta])*
                $arg:ident: $arg_type:ty
            ),*
        ) $(-> $return_type:ty)?,
    )*) => {
        #[cfg(feature = "gen-binds")]
        pub struct SeelenCommand;

        #[cfg(feature = "gen-binds")]
        impl SeelenCommand {
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

        /// Internal used as mapping of commands to their arguments
        #[cfg(feature = "gen-binds")]
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Deserialize, ts_rs::TS)]
        #[serde(rename_all_fields = "camelCase")]
        #[ts(export)]
        enum SluCmdArgumentMap {
            $(
                $fn_name {
                    $(
                        $(#[$attr])*
                        $arg: $arg_type
                    ),*
                },
            )*
        }

        /// Internal used as mapping of commands to their return types
        #[cfg(feature = "gen-binds")]
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Serialize, ts_rs::TS)]
        #[ts(export)]
        enum SluCmdReturnMap {
            $(
                $fn_name(type_or_void!($($return_type)?)),
            )*
        }

        #[macro_export]
        #[allow(clippy::crate_in_macro_def)]
        macro_rules! gen_tauri_wrappers {
            () => {
                $(
                    #[tauri::command(async)]
                    async fn $fn_name(
                        $(webview: tauri::WebviewWindow, $($with_webview)?)?
                        $($arg:$arg_type),*
                    ) -> crate::Result<$($return_type)?> {
                        $crate::__slu_command_result!(
                            $(@fallible $($is_fallible)?)?
                            Handlers::$fn_name($(webview, $($with_webview)?)? $($arg),*) $($($with_async)? .await)?
                        )
                    }
                )*
            };
        }

        #[macro_export]
        macro_rules! gen_tauri_invoke_handler {
            () => {
                tauri::generate_handler![
                    $(
                        $fn_name,
                    )*
                ]
            };
        }
    };
}

slu_commands_declaration! {
    // virtual desktops
    StateGetVirtualDesktops = get_virtual_desktops() -> VirtualDesktops,
    SwitchWorkspace =
        @fallible()
        switch_workspace(workspace_id: WorkspaceId),
    CreateWorkspace =
        @fallible()
        create_workspace(monitor_id: MonitorId) -> WorkspaceId,
    CreateWorkspaceRow =
        @fallible()
        create_workspace_row(monitor_id: MonitorId) -> WorkspaceId,
    DestroyWorkspace =
        @fallible()
        destroy_workspace(workspace_id: WorkspaceId),
    RenameWorkspace =
        @fallible()
        rename_workspace(workspace_id: WorkspaceId, name: Option<String>),
    MoveWindowToWorkspace =
        @fallible()
        move_window_to_workspace(hwnd: isize, workspace_id: WorkspaceId),

    // wallpaper
    WallpaperNext = wallpaper_next(),
    WallpaperPrev = wallpaper_prev(),
    WallpaperSaveThumbnail =
        @async()
        @fallible()
        wallpaper_save_thumbnail(wallpaper_id: WallpaperId, thumbnail_bytes: Vec<u8>),
    SetAsWallpaper =
        @webview()
        @fallible()
        set_as_wallpaper(),

    // Logging
    LogFromWebview = log_from_webview(level: u8, message: String, location: String),

    // General
    OpenFile =
        @webview()
        @fallible()
        open_file(path: String), // String is used here as explorer can open uris too
    SelectFileOnExplorer =
        @fallible()
        select_file_on_explorer(path: std::path::PathBuf),
    Run =
        @webview()
        @fallible()
        run(program: String, args: Option<RelaunchArguments>, working_dir: Option<std::path::PathBuf>, elevated: bool),
    // SimulatePerm = simulate_perm(widget_id: String, perm: String),

    IsDevMode = is_dev_mode() -> bool,
    IsAppxPackage = is_appx_package() -> bool,
    HasFixedRuntime = has_fixed_runtime() -> bool,

    GetFocusedApp = get_focused_app() -> FocusedApp,
    GetMousePosition = get_mouse_position() -> [i32; 2],
    GetKeyState =
        @fallible()
        get_key_state(key: String) -> bool,

    GetUserEnvs = get_user_envs() -> std::collections::HashMap<String, String>,
    ShowStartMenu =
        @fallible()
        show_start_menu(),
    GetIcon =
        @fallible()
        get_icon(
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional = nullable))]
        path: Option<std::path::PathBuf>,
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional = nullable))]
        umid: Option<String>
    ),
    ShowDesktop =
        @fallible()
        show_desktop(),

    RequestToUserInputShortcut =
        @webview()
        @async()
        @fallible()
        request_to_user_input_shortcut(callback_event: String),

    CheckForUpdates =
        @async()
        @fallible()
        check_for_updates() -> bool,
    // Restart the app after install the update so it returns a promise resolved with `never`
    InstallLastAvailableUpdate =
        @async()
        @fallible()
        install_last_available_update(),

    // System
    SystemGetMonitors = get_connected_monitors() -> Vec<PhysicalMonitor>,
    SetMonitorHdr =
        @fallible()
        set_monitor_hdr(id: MonitorId, state: bool),
    SystemGetColors =
        @fallible()
        get_system_colors() -> UIColors,
    SystemSetAccentColor =
        @fallible()
        set_system_accent_color(color: Color),
    SystemGetDarkMode =
        @fallible()
        get_system_dark_mode() -> bool,
    SystemSetDarkMode =
        @fallible()
        set_system_dark_mode(enabled: bool),
    SystemGetNightLightSettings =
        @fallible()
        get_system_night_light_settings() -> NightlightSettings,
    SystemGetNightLightEnabled =
        @fallible()
        get_system_night_light_enabled() -> bool,
    SystemSetNightLightEnabled =
        @fallible()
        set_system_night_light_enabled(enabled: bool),
    SystemSetNightLightColorTemperature =
        @fallible()
        set_system_night_light_color_temperature(temperature: u16),
    SystemGetLanguages = get_system_languages() -> Vec<SystemLanguage>,
    SystemSetKeyboardLayout =
        @fallible()
        set_system_keyboard_layout(id: String, handle: String),
    SystemGetImeState =
        @fallible()
        get_ime_state() -> ImeState,
    RegisterAppBar =
        @webview()
        @fallible()
        register_app_bar(rect: Rect, edge: AppBarEdge),
    UnregisterAppBar =
        @webview()
        @fallible()
        unregister_app_bar(),

    // Seelen Settings
    StateGetDefaultSettings =
        @fallible()
        state_get_default_settings() -> Settings,
    StateGetDefaultMonitorSettings = state_get_default_monitor_settings() -> MonitorConfiguration,
    StateGetDefaultWallpaperSettings = state_get_default_wallpaper_settings() -> WallpaperInstanceSettings,

    SetAutoStart =
        @fallible()
        set_auto_start(enabled: bool),
    GetAutoStartStatus =
        @fallible()
        get_auto_start_status() -> bool,
    RemoveResource =
        @async()
        @fallible()
        remove_resource(id: ResourceId, kind: ResourceKind),

    StateGetWegItems = state_get_weg_items() -> WegItems,
    StateWriteWegItems =
        @fallible()
        state_write_weg_items(items: WegItems),
    StateGetToolbarItems = state_get_toolbar_items() -> ToolbarState,
    StateWriteToolbarItems =
        @fallible()
        state_write_toolbar_items(items: ToolbarState),
    StateGetSettings =
        @fallible()
        state_get_settings(path: Option<std::path::PathBuf>) -> Settings,
    StateWriteSettings =
        @fallible()
        state_write_settings(settings: Settings),
    StateGetSettingsByApp = state_get_settings_by_app() -> Vec<AppConfig> ,
    StateGetThemes = state_get_themes() -> Vec<std::sync::Arc<Theme>>,
    StateGetPlugins = state_get_plugins() -> Vec<std::sync::Arc<Plugin>>,
    StateGetWidgets = state_get_widgets() -> Vec<std::sync::Arc<Widget>>,
    StateGetIconPacks = state_get_icon_packs() -> Vec<std::sync::Arc<IconPack>>,
    StateGetWallpapers = state_get_wallpapers() -> Vec<std::sync::Arc<Wallpaper>>,
    StateGetSystemShortcuts = state_get_system_shortcuts() -> Vec<SystemShortcutDeclaration>,
    StateSetCustomIconPack =
        @fallible()
        state_add_icon_to_custom_icon_pack(icon: IconPackEntry),
    StateDeleteCachedIcons =
        @async()
        @fallible()
        state_delete_cached_icons(),
    RegisterUserCustomAppIcon =
        @async()
        @fallible()
        register_user_custom_app_icon(icon_base64: String, entry: IconPackEntry),
    DeleteUserCustomAppIcon =
        @async()
        @fallible()
        delete_user_custom_app_icon(entry: IconPackEntry),
    StateRequestWallpaperAddition =
        @fallible()
        state_request_wallpaper_addition(),
    StateGetPerformanceMode = state_get_performance_mode() -> PerformanceMode,

    // Widgets
    DebugGetWidgetsStatuses = debug_get_widgets_statuses() -> Vec<WidgetDebugInfo>,
    DebugOpenDevTools =
        @fallible()
        debug_open_dev_tools(label: String),
    TriggerWidget =
        @fallible()
        trigger_widget(payload: WidgetTriggerPayload),
    TriggerContextMenu =
        @webview()
        @fallible()
        trigger_context_menu(menu: ContextMenu, forward_to: Option<String>),
    TriggerDialog =
        @webview()
        @fallible()
        trigger_dialog(dialog: Dialog),
    SetCurrentWidgetStatus =
        @webview()
        @fallible()
        set_current_widget_status(status: WidgetStatus),
    GetSelfWindowId =
        @webview()
        @fallible()
        get_self_window_handle() -> isize,
    SetSelfPosition =
        @webview()
        @fallible()
        set_self_position(rect: Rect),
    SetSelfZOrder =
        @webview()
        @fallible()
        set_self_z_order(z_order: ZOrder),
    WriteFile =
        @webview()
        @fallible()
        write_data_file(filename: String, content: String),
    ReadFile =
        @webview()
        @fallible()
        read_data_file(filename: String) -> String,

    // Shell
    GetNativeShellWallpaper =
        @fallible()
        get_native_shell_wallpaper() -> std::path::PathBuf,
    SetNativeShellWallpaper =
        @fallible()
        set_native_shell_wallpaper(path: std::path::PathBuf),

    // User
    GetUser = get_user() -> User,
    GetUserFolderContent = get_user_folder_content(folder_type: FolderType) -> Vec<std::path::PathBuf>,
    GetUserAppWindows = get_user_app_windows() -> Vec<UserAppWindow>,
    GetUserAppWindowsPreviews = get_user_app_windows_previews() -> std::collections::HashMap<isize, UserAppWindowPreview>,
    GetUserAppWindowsColors = get_user_app_windows_colors() -> std::collections::HashMap<isize, UserAppWindowColors>,

    // Media
    GetMediaDevices =
        @fallible()
        get_media_devices() -> (Vec<MediaDevice>, Vec<MediaDevice>),
    GetMediaSessions =
        get_media_sessions() -> Vec<MediaPlayer>,
    MediaPrev =
        @fallible()
        media_prev(id: String),
    MediaTogglePlayPause =
        @fallible()
        media_toggle_play_pause(id: String),
    MediaNext =
        @fallible()
        media_next(id: String),
    MediaSeek =
        @fallible()
        media_seek(id: String, position: i64),
    SetVolumeLevel =
        @fallible()
        set_volume_level(device_id: String, session_id: Option<String>, level: f32),
    MediaToggleMute =
        @fallible()
        media_toggle_mute(device_id: String, session_id: Option<String>),
    MediaSetDefaultDevice =
        @fallible()
        media_set_default_device(id: String, role: String),
    GetMediaWaveform =
        @fallible()
        get_media_waveform() -> AudioWaveform,

    // Brightness - Multi-monitor support
    GetAllMonitorsBrightness =
        @fallible()
        get_all_monitors_brightness() -> Vec<MonitorBrightness>,
    SetMonitorBrightness =
        @fallible()
        set_monitor_brightness(instance_name: String, level: u8),

    // Power
    GetPowerStatus = get_power_status() -> PowerStatus,
    GetPowerMode = get_power_mode() -> PowerMode,
    GetBatteries = get_batteries() -> Vec<Battery>,
    LogOut = log_out(),
    Suspend = suspend(),
    Hibernate = hibernate(),
    Restart =
        @fallible()
        restart(),
    Shutdown =
        @fallible()
        shutdown(),
    Lock =
        @fallible()
        lock(),

    // SeelenWeg
    WegCloseApp =
        @fallible()
        weg_close_app(hwnd: isize),
    WegKillApp =
        @fallible()
        weg_kill_app(hwnd: isize),
    WegToggleWindowState =
        @fallible()
        weg_toggle_window_state(hwnd: isize, was_focused: bool),
    WegPinItem =
        @fallible()
        weg_pin_item(path: std::path::PathBuf),
    WegImportPinnedTaskbarItems =
        @fallible()
        weg_import_pinned_taskbar_items() -> usize,

    // Windows Manager
    WmGetRenderTree = wm_get_render_tree() -> TwmGlobalRuntimeTree,
    SetAppWindowsPositions =
        @fallible()
        set_app_windows_positions(positions: std::collections::HashMap<isize, Rect>),
    RequestFocus =
        @fallible()
        request_focus(hwnd: isize),
    WmSetStackActiveWindow =
        @fallible()
        wm_set_stack_active_window(hwnd: isize),

    // Network
    WlanScan = wlan_scan(),
    WlanConnect =
        @fallible()
        wlan_connect(ssid: String, password: Option<String>, hidden: bool) -> bool,
    WlanDisconnect =
        @fallible()
        wlan_disconnect(),
    WlanForget =
        @fallible()
        wlan_forget(ssid: String),
    GetNetworkDefaultLocalIp =
        @fallible()
        get_network_default_local_ip() -> String,
    GetNetworkAdapters =
        @fallible()
        get_network_adapters() -> Vec<NetworkAdapter>,
    GetNetworkInternetConnection =
        @fallible()
        get_network_internet_connection() -> bool,
    GetNetworkHotspot =
        @fallible()
        get_network_hotspot() -> Option<Hotspot>,
    SetNetworkHotspotState =
        @fallible()
        set_network_hotspot_state(enabled: bool),

    // system tray
    GetSystemTrayIcons = get_system_tray_icons() -> Vec<SysTrayIcon>,
    SendSystemTrayIconAction =
        @fallible()
        send_system_tray_icon_action(id: SysTrayIconId, action: SystrayIconAction),

    // Notifications
    GetNotifications = get_notifications() -> Vec<AppNotification>,
    NotificationsClose =
        @fallible()
        notifications_close(id: u32),
    NotificationsCloseAll =
        @fallible()
        notifications_close_all(),
    ActivateNotification =
        @fallible()
        activate_notification(
        id: u32,
        umid: String,
        args: Option<String>,
        activation_type: ToastActionActivationType,
        input_data: std::collections::HashMap<String, String>
    ),

    // Radios
    GetRadios = get_radios() -> Vec<RadioDevice>,
    SetRadioState =
        @fallible()
        set_radios_state(kind: RadioDeviceKind, enabled: bool),

    // System Info
    GetSystemDisks = get_system_disks() -> Vec<Disk>,
    GetSystemNetwork = get_system_network() -> Vec<NetworkStatistics>,
    GetSystemMemory = get_system_memory() -> Memory,
    GetSystemCores = get_system_cores() -> Vec<Core>,

    // Bluetooth
    GetBluetoothDevices = get_bluetooth_devices() -> Vec<BluetoothDevice>,
    StartBluetoothScanning =
        @fallible()
        start_bluetooth_scanning(),
    StopBluetoothScanning =
        @fallible()
        stop_bluetooth_scanning(),
    RequestPairBluetoothDevice =
        @async()
        @fallible()
        request_pair_bluetooth_device(id: String) -> DevicePairingNeededAction,
    ConfirmBluetoothDevicePairing =
        @async()
        @fallible()
        confirm_bluetooth_device_pairing(id: String, answer: DevicePairingAnswer),
    DisconnectBluetoothDevice =
        @fallible()
        disconnect_bluetooth_device(id: String),
    ConnectBluetoothDevice =
        @fallible()
        connect_bluetooth_device(id: String),
    ForgetBluetoothDevice =
        @async()
        @fallible()
        forget_bluetooth_device(id: String),

    // Start Menu
    GetStartMenuItems = get_start_menu_items() -> Vec<std::sync::Arc<StartMenuItem>>,
    GetNativeStartMenu =
        @async()
        @fallible()
        get_native_start_menu() -> StartMenuLayout,

    // Trash Bin
    GetTrashBinInfo = get_trash_bin_info() -> TrashBinInfo,
    TrashBinEmpty =
        @fallible()
        trash_bin_empty(),

    // Seelen Session
    GetSeelenSession = get_seelen_session() -> Option<SeelenSession>,
    SeelenLogin =
        @fallible()
        seelen_login(),
    SeelenLogout =
        @async()
        @fallible()
        seelen_logout(),

    // Cloud Backup
    GetBackupStatus = get_backup_status() -> BackupStatus,

    // Clipboard
    ClipboardGetData = clipboard_get_data() -> ClipboardData,
    ClipboardDeleteEntry =
        @fallible()
        clipboard_delete_entry(id: String),
    ClipboardClearHistory =
        @fallible()
        clipboard_clear_history(),
    ClipboardSetContent =
        @fallible()
        clipboard_set_content(id: String),
    ClipboardPaste =
        @fallible()
        clipboard_paste(id: String, plain: bool),
    ClipboardPinEntry =
        @fallible()
        clipboard_pin_entry(id: String),
    ClipboardUnpinEntry =
        @fallible()
        clipboard_unpin_entry(id: String),

    // Fonts
    GetFonts =
        @fallible()
        get_fonts() -> Vec<SeelenFont>,

    // Focus Assist / DND
    GetFocusAssist = get_focus_assist() -> bool,
    SetFocusAssist =
        @fallible()
        set_focus_assist(enabled: bool),
    GetNotificationsMode =
        @fallible()
        get_notifications_mode() -> NotificationsMode,
    SetNotificationsMode =
        @fallible()
        set_notifications_mode(mode: NotificationsMode),
}
