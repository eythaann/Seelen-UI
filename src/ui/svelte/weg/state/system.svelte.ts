import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { SeelenWegSide } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

export const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

export const players = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, players.setByPayload);

export const notifications = lazyRune(() => invoke(SeelenCommand.GetNotifications));
subscribe(SeelenEvent.Notifications, notifications.setByPayload);

export const mousePos = lazyRune(async () => {
  const [x, y] = await invoke(SeelenCommand.GetMousePosition);
  return { x, y };
});
subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
  mousePos.value = { x, y };
});

export const trashBinInfo = lazyRune(() => invoke(SeelenCommand.GetTrashBinInfo));
subscribe(SeelenEvent.TrashBinChanged, trashBinInfo.setByPayload);

await Promise.all([
  monitors.init(),
  players.init(),
  notifications.init(),
  mousePos.init(),
  trashBinInfo.init(),
]);

export const currentMonitor = {
  get value() {
    return monitors.value.find((m) => m.id === currentMonitorId)!;
  },
};

export const mouseAtEdge = {
  get value(): SeelenWegSide | null {
    const box = currentMonitor.value.rect;
    const x = mousePos.value.x;
    const y = mousePos.value.y;

    if (x < box.left || x > box.right || y < box.top || y > box.bottom) {
      return null;
    }

    if (y === box.top) return SeelenWegSide.Top;
    if (x === box.left) return SeelenWegSide.Left;
    if (y === box.bottom - 1) return SeelenWegSide.Bottom;
    if (x === box.right - 1) return SeelenWegSide.Right;

    return null;
  },
};
