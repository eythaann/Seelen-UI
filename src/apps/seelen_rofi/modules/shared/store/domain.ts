import { LauncherHistory, SeelenLauncherSettings } from 'seelen-core';
import { IRootState } from 'src/shared.interfaces';

export interface StartMenuApp {
  label: string;
  icon: string;
  path: string;
}

export interface LauncherState extends IRootState<SeelenLauncherSettings> {
  apps: StartMenuApp[];
  history: LauncherHistory;
}
