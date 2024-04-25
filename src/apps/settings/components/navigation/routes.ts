export enum Route {
  GENERAL = 'general',
  MONITORS = 'monitors',
  SEELEN_WM = 'seelen_wm',
  SEELEN_WEG = 'seelen_weg',
  SPECIFIT_APPS = 'specifit_apps',
  SHORTCUTS = 'shortcuts',
  INFO = 'info',
}

export const WorkingInProgressRoutes = [Route.SPECIFIT_APPS, Route.MONITORS, Route.SHORTCUTS];

export const RouteLabels: Record<Route, string> = {
  [Route.GENERAL]: 'General',
  [Route.MONITORS]: 'Monitors',
  [Route.SHORTCUTS]: 'Shortcuts',
  [Route.SPECIFIT_APPS]: 'Apps Configurations',
  [Route.INFO]: 'Information',
  [Route.SEELEN_WEG]: 'SeelenWeg',
  [Route.SEELEN_WM]: 'Window Manager',
};

export const RouteExtraInfo: { [key in Route]?: string } = {
  [Route.SPECIFIT_APPS]: `
    Seelen UI use only one identifier per app (first match found) so the order in how are specificated is important,
    the lastest added will be priorized, as note the table is sorted by default from lastest to old.
  `,
};

export const RouteIcons: Record<Route, string> = {
  [Route.GENERAL]: '⚙️',
  [Route.MONITORS]: '🖥️',
  [Route.SHORTCUTS]: '🔡',
  [Route.SPECIFIT_APPS]: '🅰️',
  [Route.INFO]: '🛈',
  [Route.SEELEN_WEG]: '🚧',
  [Route.SEELEN_WM]: '🪟',
};