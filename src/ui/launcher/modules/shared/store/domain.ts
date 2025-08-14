import { SeelenLauncherSettings } from '@seelen-ui/lib/types';
import { IRootState } from 'src/ui/reduxRootState';

export interface StartMenuApp {
  path: string;
  umid: string | null;
  target: string | null;
}

export interface LauncherState extends IRootState<SeelenLauncherSettings> {
  apps: StartMenuApp[];
  history: { [key in string]?: string[] };
}
