import { IUIColors } from '@seelen-ui/lib';
import { AppConfig, Placeholder, Settings, Theme, WindowManagerLayout } from '@seelen-ui/lib/types';
export interface IRootState<T> {
  settings: T;
  colors: IUIColors;
}

export interface UserSettings {
  jsonSettings: Settings;
  yamlSettings: AppConfig[];
  themes: Theme[];
  layouts: WindowManagerLayout[];
  placeholders: Placeholder[];
  env: Record<string, string>;
  /** wallpaper url */
  wallpaper: string | null;
}
