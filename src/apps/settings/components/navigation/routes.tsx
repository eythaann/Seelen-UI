import React from 'react';

import { Icon } from '../../../shared/components/Icon';

export enum Route {
  HOME = 'home',
  GENERAL = 'general',
  SEELEN_BAR = 'seelen_bar',
  SEELEN_WM = 'seelen_wm',
  SEELEN_ROFI = 'seelen_rofi',
  SEELEN_WALL = 'seelen_wall',
  SEELEN_WEG = 'seelen_weg',
  SHORTCUTS = 'shortcuts',
  MONITORS = 'monitors',
  SPECIFIC_APPS = 'specific_apps',
  MODS = 'mods',
  DEVELOPER = 'developer',
  INFO = 'info',
}

export const RouteIcons: Record<Route, React.ReactNode> = {
  [Route.HOME]: <Icon iconName="TbHome" />,
  [Route.GENERAL]: <Icon iconName="RiSettings3Fill" />,
  [Route.MONITORS]: <Icon iconName="PiMonitorBold" />,
  [Route.SEELEN_BAR]: <Icon iconName="BiSolidDockTop" />,
  [Route.SEELEN_WEG]: <Icon iconName="BiDockBottom" />,
  [Route.SEELEN_ROFI]: <Icon iconName="MdRocketLaunch" />,
  [Route.SEELEN_WM]: <Icon iconName="BsGrid1X2Fill" size={14} />,
  [Route.SEELEN_WALL]: <Icon iconName="PiWallDuotone" />,
  [Route.SPECIFIC_APPS]: <Icon iconName="IoIosApps" />,
  [Route.SHORTCUTS]: <Icon iconName="MdLaunch" />,
  [Route.INFO]: <Icon iconName="PiInfoFill" />,
  [Route.MODS]: <Icon iconName="PiCodeBold" />,
  [Route.DEVELOPER]: <Icon iconName="PiCodeBold" />,
};
