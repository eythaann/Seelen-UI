import { lazyRune } from "libs/ui/svelte/utils";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

export const settings = lazyRune(() => invoke(SeelenCommand.StateGetSettings, { path: null }));
subscribe(SeelenEvent.StateSettingsChanged, settings.setByPayload);

export const focused = lazyRune(() => invoke(SeelenCommand.GetFocusedApp));
subscribe(SeelenEvent.GlobalFocusChanged, focused.setByPayload);

export const interactables = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

export const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

export const virtualDesktops = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, virtualDesktops.setByPayload);

export const wallpapers = lazyRune(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);

export const performanceMode = lazyRune(() => invoke(SeelenCommand.StateGetPerformanceMode));
subscribe(SeelenEvent.StatePerformanceModeChanged, performanceMode.setByPayload);

export const players = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, players.setByPayload);

await Promise.all([
  settings.init(),
  focused.init(),
  interactables.init(),
  monitors.init(),
  virtualDesktops.init(),
  wallpapers.init(),
  performanceMode.init(),
  players.init(),
]);
