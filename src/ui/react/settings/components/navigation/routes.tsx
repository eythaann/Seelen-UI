import { Icon } from "libs/ui/react/components/Icon";
import type React from "react";

export enum RoutePath {
  Home = "/",
  General = "/general",
  Resource = "/resources",
  FancyToolbar = "/widget/seelen/fancy-toolbar",
  WindowManager = "/widget/seelen/window-manager",
  WallpaperManager = "/widget/seelen/wallpaper-manager",
  SeelenWeg = "/widget/seelen/weg",
  Shortcuts = "/shortcuts",
  SettingsByMonitor = "/monitors",
  SettingsByApplication = "/specific_apps",
  DevTools = "/developer",
  IconPackEditor = "/icon_pack_editor",
  Extras = "/extras",
}

export const RouteIcons: { [key in RoutePath]?: React.ReactNode } = {
  [RoutePath.Home]: <Icon iconName="TbHome" />,
  [RoutePath.General]: <Icon iconName="RiSettings3Fill" />,
  [RoutePath.Resource]: <Icon iconName="IoColorPalette" />,
  [RoutePath.SettingsByMonitor]: <Icon iconName="PiMonitorBold" />,
  [RoutePath.SettingsByApplication]: <Icon iconName="IoIosApps" />,
  [RoutePath.Shortcuts]: <Icon iconName="MdLaunch" />,
  [RoutePath.Extras]: <Icon iconName="PiInfoFill" />,
  [RoutePath.DevTools]: <Icon iconName="PiCodeBold" />,
  [RoutePath.IconPackEditor]: <Icon iconName="PiCodeBold" />,
};
