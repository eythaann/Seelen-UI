import { SeelenLauncherSettings } from 'seelen-core';
import { IRootState } from 'src/shared.interfaces';

export interface LauncherState extends IRootState<SeelenLauncherSettings> {
  apps: any[];
}
