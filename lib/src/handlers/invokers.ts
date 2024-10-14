import { invoke as tauriInvoke, InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';

export enum SeelenCommand {
  // General
  Run = 'run',
  IsDevMode = 'is_dev_mode',
  OpenFile = 'open_file',
  RunAsAdmin = 'run_as_admin',
  SelectFileOnExplorer = 'select_file_on_explorer',
  IsVirtualDesktopSupported = 'is_virtual_desktop_supported',
  GetUserEnvs = 'get_user_envs',
  ShowAppSettings = 'show_app_settings',
  SwitchWorkspace = 'switch_workspace',
  SendKeys = 'send_keys',
  GetIcon = 'get_icon',
  GetSystemColors = 'get_system_colors',
  SimulateFullscreen = 'simulate_fullscreen',
  CheckForUpdates = 'check_for_updates',
  /** Restart the app after install the update so it returns a promise resolved with `never` */
  InstallLastAvailableUpdate = 'install_last_available_update',

  // Seelen Settings
  SetAutoStart = 'set_auto_start',
  GetAutoStartStatus = 'get_auto_start_status',
  StateGetThemes = 'state_get_themes',
  StateGetPlaceholders = 'state_get_placeholders',
  StateGetLayouts = 'state_get_layouts',
  StateGetWegItems = 'state_get_weg_items',
  StateGetSettings = 'state_get_settings',
  StateGetSpecificAppsConfigurations = 'state_get_specific_apps_configurations',
  StateGetWallpaper = 'state_get_wallpaper',
  StateSetWallpaper = 'state_set_wallpaper',
  StateGetHistory = 'state_get_history',

  // Media
  MediaPrev = 'media_prev',
  MediaTogglePlayPause = 'media_toggle_play_pause',
  MediaNext = 'media_next',
  SetVolumeLevel = 'set_volume_level',
  MediaToggleMute = 'media_toggle_mute',
  MediaSetDefaultDevice = 'media_set_default_device',

  // Brightness
  GetMainMonitorBrightness = 'get_main_monitor_brightness',
  SetMainMonitorBrightness = 'set_main_monitor_brightness',

  // Power
  LogOut = 'log_out',
  Suspend = 'suspend',
  Restart = 'restart',
  Shutdown = 'shutdown',

  // SeelenWeg
  WegCloseApp = 'weg_close_app',
  WegToggleWindowState = 'weg_toggle_window_state',
  WegRequestUpdatePreviews = 'weg_request_update_previews',
  WegPinItem = 'weg_pin_item',

  // Windows Manager
  SetWindowPosition = 'set_window_position',
  RequestFocus = 'request_focus',

  // App Launcher
  LauncherGetApps = 'launcher_get_apps',

  // Tray Icons
  TempGetByEventTrayInfo = 'temp_get_by_event_tray_info',
  OnClickTrayIcon = 'on_click_tray_icon',
  OnContextMenuTrayIcon = 'on_context_menu_tray_icon',

  // Network
  WlanGetProfiles = 'wlan_get_profiles',
  WlanStartScanning = 'wlan_start_scanning',
  WlanStopScanning = 'wlan_stop_scanning',
  WlanConnect = 'wlan_connect',
  WlanDisconnect = 'wlan_disconnect',

  // Notifications
  NotificationsClose = 'notifications_close',
  NotificationsCloseAll = 'notifications_close_all',
}

type ReturnTypeByCommand = Record<SeelenCommand, unknown> & {
  [SeelenCommand.CheckForUpdates]: boolean;
  [SeelenCommand.InstallLastAvailableUpdate]: never;
};

export type SeelenCommandReturn<T extends SeelenCommand> = ReturnTypeByCommand[T];

export async function invoke<T extends SeelenCommand>(
  command: T,
  args?: InvokeArgs,
  options?: InvokeOptions,
): Promise<SeelenCommandReturn<T>> {
  return tauriInvoke(command, args, options);
}
