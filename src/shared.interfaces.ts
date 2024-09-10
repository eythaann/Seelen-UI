import { UIColors } from 'seelen-core';
import { Settings, Theme } from 'seelen-core';

import { AppConfiguration } from './apps/settings/modules/appsConfigurations/domain';
import { Layout, LayoutSchema, NoFallbackBehavior } from './apps/shared/schemas/Layout';
import { Placeholder } from './apps/shared/schemas/Placeholders';
export interface IRootState<T> {
  settings: T;
  colors: UIColors;
}

export interface UserSettings {
  jsonSettings: Settings;
  yamlSettings: AppConfiguration[];
  themes: Theme[];
  layouts: Layout[];
  placeholders: Placeholder[];
  env: Record<string, string>;
  /** wallpaper url */
  wallpaper: string | null;
}

const _defaultLayout = LayoutSchema.parse({});
export const defaultLayout: Layout = {
  ..._defaultLayout,
  info: {
    ..._defaultLayout.info,
    filename: 'unknown',
  },
  noFallbackBehavior: NoFallbackBehavior.Unmanaged,
};
