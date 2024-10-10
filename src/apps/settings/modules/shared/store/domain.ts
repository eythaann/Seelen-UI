import { AppConfiguration, Settings, Theme, UIColors, WindowManagerLayout } from 'seelen-core';
import { Placeholder } from 'seelen-core';

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
}
