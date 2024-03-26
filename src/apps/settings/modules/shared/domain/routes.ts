export enum Route {
  GENERAL = 'general',
  MONITORS = 'monitors',
  STYLES = 'styles',
  SEELEN_WEG = 'seelen_weg',
  SPECIFIT_APPS = 'specifit_apps',
  SHORTCUTS = 'shortcuts',
  UPDATES = 'updates',
  INFO = 'info',
}

export const RouteLabels: Record<Route, string> = {
  [Route.GENERAL]: 'General',
  [Route.MONITORS]: 'Monitors',
  [Route.STYLES]: 'Visuals',
  [Route.SHORTCUTS]: 'Shortcuts',
  [Route.SPECIFIT_APPS]: 'Apps Configurations',
  [Route.INFO]: 'Information',
  [Route.UPDATES]: 'Updates',
  [Route.SEELEN_WEG]: 'SeelenWeg Beta',
};

export const RouteExtraInfo: { [key in Route]?: string } = {
  [Route.SPECIFIT_APPS]: `
    Komorebi-UI use only one identifier per app (first match found) so the order in how are specificated is important,
    the lastest added will be priorized, as note the table is sorted by default from lastest to old.
  `,
};

export const RouteIcons: Record<Route, string> = {
  [Route.GENERAL]: '⚙️',
  [Route.MONITORS]: '🖥️',
  [Route.STYLES]: '🖼️',
  [Route.SHORTCUTS]: '🔡',
  [Route.SPECIFIT_APPS]: '🅰️',
  [Route.INFO]: '🛈',
  [Route.UPDATES]: '🔄️',
  [Route.SEELEN_WEG]: '🚧',
};