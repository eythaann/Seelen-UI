export enum ToolbarModuleType {
  Generic = 'generic',
  Text = 'text',
  Date = 'date',
  Power = 'power',
  Settings = 'settings',
  Network = 'network',
  Workspaces = 'workspaces',
  Media = 'media',
  Tray = 'tray',
  Device = 'device',
  Notifications = 'notifications',
}

export enum WorkspaceTMMode {
  Dotted = 'dotted',
  Named = 'named',
  Numbered = 'numbered',
}

export enum TimeUnit {
  SECOND = 'second',
  MINUTE = 'minute',
  HOUR = 'hour',
  DAY = 'day',
}

export enum DeviceTMSubType {
  Disk = 'disk',
  CPU = 'cpu',
  Memory = 'memory',
}

export interface BaseToolbarModule {
  id: string;
  type: ToolbarModuleType;
  template: string;
  tooltip: string | null;
  badge: string | null;
  /** @deprecated, use `onClickV2` instead */
  onClick: string | null;
  onClickV2: string | null;
  style: Record<string, any>;
}

export interface GenericToolbarModule extends BaseToolbarModule {
  type: ToolbarModuleType.Generic | ToolbarModuleType.Text;
}

export interface TrayTM extends BaseToolbarModule {
  type: ToolbarModuleType.Tray;
}

export interface DateToolbarModule extends BaseToolbarModule {
  type: ToolbarModuleType.Date;
  /** @deprecated v2 uses settings date format instead (it will perform the minimal updates) */
  each: TimeUnit;
  /** @deprecated v2 uses settings date format instead */
  format: string;
}

export interface PowerToolbarModule extends BaseToolbarModule {
  type: ToolbarModuleType.Power;
}

export interface NetworkTM extends BaseToolbarModule {
  type: ToolbarModuleType.Network;
  withWlanSelector: boolean;
}

export interface MediaTM extends BaseToolbarModule {
  type: ToolbarModuleType.Media;
  withMediaControls: boolean;
}

export interface NotificationsTM extends BaseToolbarModule {
  type: ToolbarModuleType.Notifications;
}

export interface DeviceTM extends BaseToolbarModule {
  type: ToolbarModuleType.Device;
}

export interface SettingsToolbarModule extends BaseToolbarModule {
  type: ToolbarModuleType.Settings;
}

export interface WorkspacesTM extends BaseToolbarModule {
  type: ToolbarModuleType.Workspaces;
  mode: WorkspaceTMMode;
}

export type ToolbarModule =
  | GenericToolbarModule
  | DateToolbarModule
  | PowerToolbarModule
  | SettingsToolbarModule
  | WorkspacesTM
  | TrayTM
  | NetworkTM
  | MediaTM
  | DeviceTM
  | NotificationsTM;

export interface CreatorInfo {
  displayName: string;
  author: string;
  description: string;
  filename: string;
}

export interface Placeholder {
  info: CreatorInfo;
  left: ToolbarModule[];
  center: ToolbarModule[];
  right: ToolbarModule[];
}
