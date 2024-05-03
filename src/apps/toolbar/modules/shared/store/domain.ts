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

export interface RootState extends IRootState<FancyToolbar> {
  focused: ActiveApp | null;
  placeholder: Placeholder | null;
  env: Record<string, string>;
  powerStatus: PowerStatus;
}