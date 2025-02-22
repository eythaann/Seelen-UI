import { SystemLanguage } from '@seelen-ui/lib';
import {
  FancyToolbarSettings,
  File,
  Placeholder,
  Plugin,
  Settings,
  User,
} from '@seelen-ui/lib/types';
import { SoftOpaque } from 'readable-types';

import { WlanBssEntry } from '../../network/domain';
import { AppNotification } from '../../Notifications/domain';

import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';

/** https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status */
export interface PowerStatus {
  acLineStatus: number;
  batteryFlag: number;
  batteryLifePercent: number;
  systemStatusFlag: number;
  batteryLifeTime: number;
  batteryFullLifeTime: number;
}

export enum PowerPlan {
  BatterySaver = 'BatterySaver',
  Balanced = 'Balanced',
  BetterBattery = 'BetterBattery',
  HighPerformance = 'HighPerformance',
  MaxPerformance = 'MaxPerformance',
  GameMode = 'GameMode',
  MixedReality = 'MixedReality',
  Unknown = 'Unknown',
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
  label: string;
  registry: {
    key: string;
    executablePath: string;
    initialTooltip: string | null;
    /** cached PNG buffer */
    iconSnapshot: number[] | null;
    iconGuid: string | null;
    iconUid: number | null;
    isPromoted: boolean;
    isRunning: boolean;
  };
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
  umid: string;
  title: string;
  author: string;
  thumbnail: string | null;
  playing: boolean;
  default: boolean;
  owner: {
    name: string;
  };
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

export type WorkspaceId = SoftOpaque<string, 'WorkspaceId'>;
export interface Workspace {
  id: WorkspaceId;
  name: string | null;
}

export interface RootState extends IRootState<FancyToolbarSettings>, Pick<Settings, 'dateFormat'> {
  version: number;
  items: Placeholder;
  plugins: Plugin[];
  isOverlaped: boolean;
  user: User | null;
  userRecentFolder: File[];
  userDesktopFolder: File[];
  userDocumentsFolder: File[];
  userDownloadsFolder: File[];
  userPicturesFolder: File[];
  userVideosFolder: File[];
  userMusicFolder: File[];
  focused: FocusedApp | null;
  env: Record<string, string>;
  powerStatus: PowerStatus;
  powerPlan: PowerPlan;
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
  languages: SystemLanguage[];
}
