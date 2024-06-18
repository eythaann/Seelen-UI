import { Icon } from '../../../shared/components/Icon';
import React from 'react';

export enum Route {
  GENERAL = 'general',
  SEELEN_BAR = 'seelen_bar',
  SEELEN_WM = 'seelen_wm',
  SEELEN_WEG = 'seelen_weg',
  MONITORS = 'monitors',
  SPECIFIT_APPS = 'specifit_apps',
  SHORTCUTS = 'shortcuts',
  INFO = 'info',
}

export const WorkingInProgressRoutes = [Route.MONITORS];

export const RouteLabels: Record<Route, string> = {
  [Route.GENERAL]: 'General',
  [Route.MONITORS]: 'Monitors',
  [Route.SHORTCUTS]: 'Shortcuts',
  [Route.SPECIFIT_APPS]: 'Apps Configurations',
  [Route.INFO]: 'Information',
  [Route.SEELEN_WEG]: 'SeelenWeg',
  [Route.SEELEN_WM]: 'Window Manager',
  [Route.SEELEN_BAR]: 'Fancy Toolbar',
};

export const RouteExtraInfo: { [key in Route]?: string } = {
  [Route.SPECIFIT_APPS]: `
    Seelen UI use only one identifier per app (first match found) so the order in how are specificated is important,
    the latest added will be priorized, as note the table is sorted by default from latest to old.
  `,
};

export const RouteIcons: Record<Route, React.ReactNode> = {
  [Route.GENERAL]: <Icon iconName="RiSettings3Fill" />,
  [Route.MONITORS]: <Icon iconName="PiMonitorBold" />,
  [Route.SEELEN_BAR]: <Icon iconName="BiSolidDockTop" />,
  [Route.SEELEN_WM]: <Icon iconName="BsGrid1X2Fill" propsIcon={{ color: '#0086b4', size: 14 }} />,
  [Route.SEELEN_WEG]: <Icon iconName="BiDockBottom" />,
  [Route.SPECIFIT_APPS]: <Icon iconName="IoIosApps" propsIcon={{ color: '#d71913' }} />,
  [Route.SHORTCUTS]: '🔡',
  [Route.INFO]: <Icon iconName="PiInfoFill" />,
};