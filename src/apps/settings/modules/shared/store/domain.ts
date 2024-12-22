import { ConnectedMonitor, IUIColors } from '@seelen-ui/lib';
import { AppConfig, IconPack, Placeholder, Plugin, Profile, Settings, Theme, Widget, WindowManagerLayout } from '@seelen-ui/lib/types';

import { Route } from '../../../components/navigation/routes';

export interface RootState extends Settings {
  lastLoaded: this | null;
  route: Route;
  toBeSaved: boolean;
  toBeRestarted: boolean;
  appsConfigurations: AppConfig[];
  availableThemes: Theme[];
  availableIconPacks: IconPack[];
  availableLayouts: WindowManagerLayout[];
  availablePlaceholders: Placeholder[];
  autostart: boolean | null;
  colors: IUIColors;
  plugins: Plugin[];
  widgets: Widget[];
  profiles: Profile[];
  connectedMonitors: ConnectedMonitor[];
}
