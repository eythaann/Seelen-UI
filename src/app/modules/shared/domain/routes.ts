export enum Route {
  GENERAL = 'general',
  MONITORS = 'monitors',
  STYLES = 'styles',
  SPECIFIT_APPS = 'specifit_apps',
  SHORTCUTS = 'shortcuts',
  INFO = 'info',
}

export const RouteLabels: Record<Route, string> = {
  [Route.GENERAL]: 'General',
  [Route.MONITORS]: 'Monitors',
  [Route.STYLES]: 'Visuals',
  [Route.SHORTCUTS]: 'Shortcuts',
  [Route.SPECIFIT_APPS]: 'Apps Configurations',
  [Route.INFO]: 'Information',
};

export const RouteExtraInfo: Record<Route, string | null> = {
  [Route.GENERAL]: null,
  [Route.MONITORS]: null,
  [Route.STYLES]: null,
  [Route.SHORTCUTS]: null,
  [Route.SPECIFIT_APPS]: `
    Komorebi-UI use only one identifier per app (first match found) so the order in how are specificated is important,
    the lastest added will be priorized, as note the table is sorted by default from lastest to old.
  `,
  [Route.INFO]: null,
};

export const RouteIcons: Record<Route, string> = {
  [Route.GENERAL]: '⚙️',
  [Route.MONITORS]: '🖥️',
  [Route.STYLES]: '🖼️',
  [Route.SHORTCUTS]: '🔡',
  [Route.SPECIFIT_APPS]: '🅰️',
  [Route.INFO]: '🛈',
};