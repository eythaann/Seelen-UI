import { computed, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { SeelenWegSide } from "@seelen-ui/lib/types";
import { lazySignal } from "@shared/LazySignal";

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

export const $monitors = signal(await invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, (e) => {
  $monitors.value = e.payload;
});

export const $current_monitor = computed(
  () => $monitors.value.find((m) => m.id === currentMonitorId)!,
);

export const $players = lazySignal(() => invoke(SeelenCommand.GetMediaSessions));
await subscribe(SeelenEvent.MediaSessions, $players.setByPayload);
await $players.init();

export const $notifications = lazySignal(() => invoke(SeelenCommand.GetNotifications));
await subscribe(SeelenEvent.Notifications, $notifications.setByPayload);
await $notifications.init();

export const $mouse_pos = lazySignal(async () => {
  const [x, y] = await invoke(SeelenCommand.GetMousePosition);
  return { x, y };
});
await subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
  $mouse_pos.value = { x, y };
});
await $mouse_pos.init();

export const $mouse_at_edge = computed<SeelenWegSide | null>(() => {
  const box = $current_monitor.value.rect;
  const x = $mouse_pos.value.x;
  const y = $mouse_pos.value.y;

  if (x < box.left || x > box.right || y < box.top || y > box.bottom) {
    return null;
  }

  if (y === box.top) {
    return SeelenWegSide.Top;
  }

  if (x === box.left) {
    return SeelenWegSide.Left;
  }

  if (y === box.bottom - 1) {
    return SeelenWegSide.Bottom;
  }

  if (x === box.right - 1) {
    return SeelenWegSide.Right;
  }
  return null;
});
