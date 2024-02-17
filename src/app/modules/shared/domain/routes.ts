export enum Route {
  GENERAL = 'general',
  MONITORS = 'monitors',
  SPECIFIT_APPS = 'specifit_apps',
  SHORTCUTS = 'shortcuts',
  INFO = 'info',
}

export const RouteLabels: Record<Route, string> = {
  [Route.GENERAL]: 'General',
  [Route.MONITORS]: 'Monitors',
  [Route.SHORTCUTS]: 'Shortcuts',
  [Route.SPECIFIT_APPS]: 'Apps Configurations',
  [Route.INFO]: 'Information',
};

export const RouteIcons: Record<Route, string> = {
  [Route.GENERAL]: '⚙️',
  [Route.MONITORS]: '🖥️',
  [Route.SHORTCUTS]: '🔡',
  [Route.SPECIFIT_APPS]: '🅰️',
  [Route.INFO]: '🛈',
};