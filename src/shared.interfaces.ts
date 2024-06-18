import { Layout, LayoutSchema, NoFallbackBehavior } from './apps/shared/schemas/Layout';
import { Placeholder } from './apps/shared/schemas/Placeholders';
import { ISettings } from './apps/shared/schemas/Settings';
import { Theme, ThemeSchema } from './apps/shared/schemas/Theme';

export interface IRootState<T> {
  settings: T;
  themeLayers: Theme['layers'];
}

export interface UserSettings {
  jsonSettings: ISettings;
  yamlSettings: anyObject[];
  themes: Theme[];
  bgLayers: Theme['layers'];
  layouts: Layout[];
  placeholders: Placeholder[];
  env: Record<string, string>;
}

export interface AppTemplate {
  name: string;
  description: string;
  apps: anyObject[];
}

const _defaultTheme = ThemeSchema.parse({});
export const defaultTheme: Theme = {
  ..._defaultTheme,
  info: {
    ..._defaultTheme.info,
    filename: 'unknown',
  },
};

const _defaultLayout = LayoutSchema.parse({});
export const defaultLayout: Layout = {
  ..._defaultLayout,
  info: {
    ..._defaultLayout.info,
    filename: 'unknown',
  },
  noFallbackBehavior: NoFallbackBehavior.Unmanaged,
};