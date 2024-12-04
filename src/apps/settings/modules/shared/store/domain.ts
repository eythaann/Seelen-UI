import {
  AppConfiguration,
  Placeholder,
  Plugin,
  Profile,
  Settings,
  Theme,
  UIColors,
  Widget,
  WindowManagerLayout,
} from 'seelen-core';

import { Route } from '../../../components/navigation/routes';

export interface RootState extends Settings {
  lastLoaded: this | null;
  route: Route;
  toBeSaved: boolean;
  toBeRestarted: boolean;
  appsConfigurations: AppConfiguration[];
  availableThemes: Theme[];
  availableLayouts: WindowManagerLayout[];
  availablePlaceholders: Placeholder[];
  autostart: boolean | null;
  wallpaper: string | null;
  colors: UIColors;
  plugins: Plugin[];
  widgets: Widget[];
  profiles: Profile[];
}
