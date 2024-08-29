import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';
import { FancyToolbar } from '../../../../shared/schemas/FancyToolbar';
import { Placeholder } from '../../../../shared/schemas/Placeholders';
import { SoftOpaque } from 'readable-types';

import { WlanBssEntry } from '../../network/domain';

/** https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status */
export interface PowerStatus {
  acLineStatus: number;
  batteryFlag: number;
  batteryLifePercent: number;
  systemStatusFlag: number;
  batteryLifeTime: number;
  batteryFullLifeTime: number;
}

export interface Battery {
  // Static info
  vendor: string | null;
  model: string | null;
  serialNumber: string | null;
  technology: string;

  // Common information
  state: string;
  capacity: number;
  temperature: number | null;
  percentage: number;
  cycleCount: number | null;
  smartCharging: boolean;

  // Energy stats
  energy: number;
  energyFull: number;
  energyFullDesign: number;
  energyRate: number;
  voltage: number;

  // Charge stats
  timeToFull: number | null;
  timeToEmpty: number | null;
}

export interface TrayInfo {
  label: string | null;
  icon: string | null;
}

export interface NetworkAdapter {
  name: string;
  description: string;
  status: 'up' | 'down';
  dnsSuffix: string;
  type: string;
  gateway: string | null;
  mac: string;
  ipv4: string | null;
  ipv6: string | null;
}

export interface MediaChannelTransportData {
  id: string;
  title: string;
  author: string;
  thumbnail: string | null;
  playing: boolean;
  default: boolean;
}

export interface MediaDeviceChannel {
  id: string;
  instance_id: string;
  process_id: number;
  name: string;
  icon_path: string | null;
  is_system: boolean;
  volume: number;
  muted: boolean;
}

export interface MediaDevice {
  id: string;
  name: string;
  is_default_multimedia: boolean;
  is_default_communications: boolean;
  sessions: MediaDeviceChannel[];
  volume: number;
  muted: boolean;
}

export interface AppNotification {
  id: number;
  app_name: string;
  app_description: string;
  app_logo: string | null;
  body: string[];
  date: number;
}

export interface UIColors {
  background: string;
  foreground: string;
  accent_darkest: string;
  accent_darker: string;
  accent_dark: string;
  accent: string;
  accent_light: string;
  accent_lighter: string;
  accent_lightest: string;
  complement: string | null;
}

export type WorkspaceId = SoftOpaque<string, 'WorkspaceId'>;
export interface Workspace {
  id: WorkspaceId;
  name: string | null;
}

export interface RootState extends IRootState<FancyToolbar> {
  version: number;
  isOverlaped: boolean;
  focused: FocusedApp | null;
  placeholder: Placeholder | null;
  env: Record<string, string>;
  powerStatus: PowerStatus;
  batteries: Battery[];
  workspaces: Workspace[];
  activeWorkspace: WorkspaceId | null;
  systemTray: TrayInfo[];
  networkAdapters: NetworkAdapter[];
  networkLocalIp: string | null;
  online: boolean;
  wlanBssEntries: WlanBssEntry[];
  mediaSessions: MediaChannelTransportData[];
  mediaOutputs: MediaDevice[];
  mediaInputs: MediaDevice[];
  notifications: AppNotification[];
  colors: UIColors;
}
