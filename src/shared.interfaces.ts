import {
  AppConfiguration,
  Placeholder,
  Settings,
  Theme,
  UIColors,
  WindowManagerLayout,
} from 'seelen-core';
export interface IRootState<T> {
  settings: T;
  colors: UIColors;
}

export interface UserSettings {
  jsonSettings: Settings;
  yamlSettings: AppConfiguration[];
  themes: Theme[];
  layouts: WindowManagerLayout[];
  placeholders: Placeholder[];
  env: Record<string, string>;
  /** wallpaper url */
  wallpaper: string | null;
}
