import React from 'react';

import { Icon } from '../../../shared/components/Icon';

export enum RoutePath {
  Home = '/',
  General = '/general',
  Resource = '/resources',
  FancyToolbar = '/seelen_bar',
  WindowManager = '/seelen_wm',
  AppLauncher = '/seelen_rofi',
  WallpaperManager = '/seelen_wall',
  SeelenWeg = '/seelen_weg',
  Shortcuts = '/shortcuts',
  SettingsByMonitor = '/monitors',
  SettingsByApplication = '/specific_apps',
  Mods = '/mods',
  DevTools = '/developer',
  Extras = '/extras',
}

export const RouteIcons: Record<RoutePath, React.ReactNode> = {
  [RoutePath.Home]: <Icon iconName="TbHome" />,
  [RoutePath.General]: <Icon iconName="RiSettings3Fill" />,
  [RoutePath.Resource]: <Icon iconName="IoColorPalette" />,
  [RoutePath.SettingsByMonitor]: <Icon iconName="PiMonitorBold" />,
  [RoutePath.FancyToolbar]: <Icon iconName="BiSolidDockTop" />,
  [RoutePath.SeelenWeg]: <Icon iconName="BiDockBottom" />,
  [RoutePath.AppLauncher]: <Icon iconName="MdRocketLaunch" />,
  [RoutePath.WindowManager]: <Icon iconName="BsGrid1X2Fill" size={14} />,
  [RoutePath.WallpaperManager]: <Icon iconName="PiWallDuotone" />,
  [RoutePath.SettingsByApplication]: <Icon iconName="IoIosApps" />,
  [RoutePath.Shortcuts]: <Icon iconName="MdLaunch" />,
  [RoutePath.Extras]: <Icon iconName="PiInfoFill" />,
  [RoutePath.Mods]: <Icon iconName="PiCodeBold" />,
  [RoutePath.DevTools]: <Icon iconName="PiCodeBold" />,
};
