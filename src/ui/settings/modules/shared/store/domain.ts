import {
  AppConfig,
  FancyToolbarSettings,
  IconPack,
  PhysicalMonitor,
  Plugin,
  Profile,
  SeelenLauncherSettings,
  SeelenWallSettings,
  SeelenWegSettings,
  Settings,
  Theme,
  UIColors,
  Wallpaper,
  Widget,
  WindowManagerSettings,
} from "@seelen-ui/lib/types";

export interface RootState extends Settings {
  lastLoaded: this | null;
  toBeSaved: boolean;
  toBeRestarted: boolean;
  appsConfigurations: AppConfig[];
  availableThemes: Theme[];
  availableIconPacks: IconPack[];
  autostart: boolean | null;
  colors: UIColors;
  plugins: Plugin[];
  widgets: Widget[];
  wallpapers: Wallpaper[];
  profiles: Profile[];
  connectedMonitors: PhysicalMonitor[];
  // migrated since v2.1.0 check src\apps\settings\modules\shared\store\app\StateBridge.ts
  fancyToolbar: FancyToolbarSettings;
  seelenweg: SeelenWegSettings;
  wall: SeelenWallSettings;
  launcher: SeelenLauncherSettings;
  windowManager: WindowManagerSettings;
}
