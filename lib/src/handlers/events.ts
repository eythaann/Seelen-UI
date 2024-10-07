export enum SeelenEvent {
  WorkspacesChanged = 'workspaces-changed',
  ActiveWorkspaceChanged = 'active-workspace-changed',

  GlobalFocusChanged = 'global-focus-changed',
  GlobalMouseMove = 'global-mouse-move',

  HandleLayeredHitboxes = 'handle-layered',

  MediaSessions = 'media-sessions',
  MediaInputs = 'media-inputs',
  MediaOutputs = 'media-outputs',

  NetworkDefaultLocalIp = 'network-default-local-ip',
  NetworkAdapters = 'network-adapters',
  NetworkInternetConnection = 'network-internet-connection',
  NetworkWlanScanned = 'wlan-scanned',

  Notifications = 'notifications',

  PowerStatus = 'power-status',
  BatteriesStatus = 'batteries-status',

  ColorsChanged = 'colors-changed',

  TrayInfo = 'tray-info',

  ToolbarOverlaped = 'set-auto-hide',

  WegOverlaped = 'set-auto-hide',
  WegSetFocusedHandle = 'set-focused-handle',
  WegSetFocusedExecutable = 'set-focused-executable',
  WegUpdateOpenAppInfo = 'update-open-app-info',
  WegAddOpenApp = 'add-open-app',
  WegRemoveOpenApp = 'remove-open-app',

  WMSetReservation = 'set-reservation',
  WMUpdateHeight = 'update-height',
  WMUpdateWidth = 'update-width',
  WMResetWorkspaceSize = 'reset-workspace-size',
  WMFocus = 'focus',
  WMSetActiveWorkspace = 'set-active-workspace',
  WMAddWindow = 'add-window',
  WMUpdateWindow = 'update-window',
  WMRemoveWindow = 'remove-window',

  WMForceRetiling = 'wm-force-retiling',
  WMSetLayout = 'wm-set-layout',
  WMSetOverlayVisibility = 'wm-set-overlay-visibility',
  WMSetActiveWindow = 'wm-set-active-window',

  WallStop = 'wall-stop',

  StateSettingsChanged = 'settings-changed',
  StateWegItemsChanged = 'weg-items',
  StateThemesChanged = 'themes',
  StatePlaceholdersChanged = 'placeholders',
  StateLayoutsChanged = 'layouts',
  StateSettingsByAppChanged = 'settings-by-app',
  StateHistoryChanged = 'history',
  StateIconPacksChanged = 'icon-packs',
}
