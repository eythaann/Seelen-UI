import { invoke, PluginList, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { UserAppWindowColors } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

export const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

export const virtualDesktops = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, virtualDesktops.setByPayload);

export const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

export const players = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, players.setByPayload);

export const mousePos = lazyRune(async () => {
  const [x, y] = await invoke(SeelenCommand.GetMousePosition);
  return { x, y };
});
subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
  mousePos.value = { x, y };
});

export const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));

export const selfWinId = lazyRune(() => invoke(SeelenCommand.GetSelfWindowId));

export const interactables = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

const trayIcons = lazyRune(() =>
  invoke(SeelenCommand.GetSystemTrayIcons).catch((error) => {
    console.error(error);
    return [];
  })
);
const traySubscription = subscribe(SeelenEvent.SystemTrayChanged, trayIcons.setByPayload);

class TrayState {
  get icons() {
    return trayIcons.value;
  }
}

export const trayState = new TrayState();

export const windowsColors = lazyRune<Record<string, UserAppWindowColors>>(
  () => invoke(SeelenCommand.GetUserAppWindowsColors),
);
subscribe(SeelenEvent.UserAppWindowsColorsChanged, windowsColors.setByPayload);

export const focused = lazyRune(() => invoke(SeelenCommand.GetFocusedApp));

export const widgetStatuses = lazyRune(() => invoke(SeelenCommand.DebugGetWidgetsStatuses));
subscribe(SeelenEvent.WidgetDebugInfoChanged, widgetStatuses.setByPayload);

export const wegItems = lazyRune(() => invoke(SeelenCommand.StateGetWegItems));

export const plugins = lazyRune(async () => (await PluginList.getAsync()).forCurrentWidget());
PluginList.onChange((list) => {
  plugins.value = list.forCurrentWidget();
});

await Promise.all([
  virtualDesktops.init(),
  monitors.init(),
  players.init(),
  mousePos.init(),
  settings.init(),
  selfWinId.init(),
  interactables.init(),
  traySubscription.then(() => trayIcons.init()),
  windowsColors.init(),
  focused.init(),
  widgetStatuses.init(),
  wegItems.init(),
  plugins.init(),
]);
