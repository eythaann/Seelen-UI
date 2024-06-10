import { IRootState } from '../../../../../shared.interfaces';
import { FancyToolbar } from '../../../../utils/schemas/FancyToolbar';
import { Placeholder } from '../../../../utils/schemas/Placeholders';

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

export interface TrayInfo {
  label: string;
  icon: string | null;
}

export interface RootState extends IRootState<FancyToolbar> {
  focused: ActiveApp | null;
  placeholder: Placeholder | null;
  env: Record<string, string>;
  powerStatus: PowerStatus;
  workspaces: string[];
  activeWorkspace: number;
  systemTray: TrayInfo[];
}