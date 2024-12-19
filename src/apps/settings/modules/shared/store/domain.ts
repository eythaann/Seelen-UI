import { ConnectedMonitor, IUIColors } from '@seelen-ui/lib';
import { AppConfig, Placeholder, Plugin, Profile, Settings, Theme, Widget, WindowManagerLayout } from '@seelen-ui/lib/types';

import { Route } from '../../../components/navigation/routes';

export interface RootState extends Settings {
  lastLoaded: this | null;
  route: Route;
  toBeSaved: boolean;
  toBeRestarted: boolean;
  appsConfigurations: AppConfig[];
  availableThemes: Theme[];
  availableLayouts: WindowManagerLayout[];
  availablePlaceholders: Placeholder[];
  autostart: boolean | null;
  wallpaper: string | null;
  colors: IUIColors;
  plugins: Plugin[];
  widgets: Widget[];
  profiles: Profile[];
  connectedMonitors: ConnectedMonitor[];
}
