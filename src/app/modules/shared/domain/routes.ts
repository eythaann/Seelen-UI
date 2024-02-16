export enum Route {
  GENERAL = 'general',
  MONITORS = 'monitors',
  SPECIFIT_APPS = 'specifit_apps',
  SHORTCUTS = 'shortcuts',
}

export const RouteLabels: Record<Route, string> = {
  [Route.GENERAL]: 'General',
  [Route.MONITORS]: 'Monitors',
  [Route.SHORTCUTS]: 'Shourcuts',
  [Route.SPECIFIT_APPS]: 'Apps Configurations',
};