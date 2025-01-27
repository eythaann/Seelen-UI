import { ConnectedMonitor, IUIColors } from '@seelen-ui/lib';
import {
  AppConfig,
  FancyToolbarSettings,
  IconPack,
  Plugin,
  Profile,
  SeelenLauncherSettings,
  SeelenWallSettings,
  SeelenWegSettings,
  Settings,
  Theme,
  Widget,
  WindowManagerSettings,
} from '@seelen-ui/lib/types';

import { Route } from '../../../components/navigation/routes';

export interface RootState extends Settings {
  lastLoaded: this | null;
  route: Route;
  toBeSaved: boolean;
  toBeRestarted: boolean;
  appsConfigurations: AppConfig[];
  availableThemes: Theme[];
  availableIconPacks: IconPack[];
  autostart: boolean | null;
  colors: IUIColors;
  plugins: Plugin[];
  widgets: Widget[];
  profiles: Profile[];
  connectedMonitors: ConnectedMonitor[];
  // migrated since v2.1.0
  fancyToolbar: FancyToolbarSettings;
  seelenweg: SeelenWegSettings;
  wall: SeelenWallSettings;
  launcher: SeelenLauncherSettings;
  windowManager: WindowManagerSettings;
}
