import { IRootState } from 'src/shared.interfaces';

export interface LauncherState extends IRootState<{}> {
  apps: any[];
}
