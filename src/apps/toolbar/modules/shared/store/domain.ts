import { IRootState } from '../../../../../shared.interfaces';
import { FancyToolbar } from '../../../../shared/schemas/FancyToolbar';
import { Placeholder } from '../../../../shared/schemas/Placeholders';

import { WlanBssEntry } from '../../network/domain';

export interface ActiveApp {
  name: string;
  title: string;
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

export interface RootState extends IRootState<FancyToolbar> {
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
  accentColor: string;
  wlanBssEntries: WlanBssEntry[];
}