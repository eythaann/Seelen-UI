import { Layout, LayoutSchema, NoFallbackBehavior } from './apps/shared/schemas/Layout';
import { Placeholder } from './apps/shared/schemas/Placeholders';
import { ISettings } from './apps/shared/schemas/Settings';
import { Theme } from './apps/shared/schemas/Theme';

import { AppConfiguration } from './apps/settings/modules/appsConfigurations/domain';

export interface IRootState<T> {
  settings: T;
}

export interface UserSettings {
  jsonSettings: ISettings;
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
