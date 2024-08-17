import { IRootState } from '../../../../../shared.interfaces';
import { FancyToolbar } from '../../../../shared/schemas/FancyToolbar';
import { Placeholder } from '../../../../shared/schemas/Placeholders';

import { WlanBssEntry } from '../../network/domain';

export interface ActiveApp {
  name: string;
  title: string;
  exe: string | null;
}

export interface PowerStatus {
  ACLineStatus: number;
  BatteryFlag: number;
  BatteryLifePercent: number;
  SystemStatusFlag: number;
  BatteryLifeTime: number;
  BatteryFullLifeTime: number;
}

export interface Battery {
  percent: number;
}

export interface TrayInfo {
  label: string;
  icon: string | null;
}

export interface NetworkAdapter {
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

export interface RootState extends IRootState<FancyToolbar> {
  version: number;
  focused: ActiveApp | null;
  placeholder: Placeholder | null;
  env: Record<string, string>;
  powerStatus: PowerStatus;
  batteries: Battery[];
  workspaces: string[];
  activeWorkspace: number;
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
