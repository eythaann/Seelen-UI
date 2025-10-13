import { computed, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide } from "@seelen-ui/lib/types";

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

const initialDesktops = await invoke(SeelenCommand.StateGetVirtualDesktops);
export const $virtual_desktop = signal(
  initialDesktops.monitors[currentMonitorId],
);
subscribe(SeelenEvent.VirtualDesktopsChanged, (e) => {
  $virtual_desktop.value = e.payload.monitors[currentMonitorId];
});

const $monitors = signal(await invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, (e) => {
  $monitors.value = e.payload;
});

const $current_monitor = computed(() => $monitors.value.find((m) => m.id === currentMonitorId)!);

const $mouse_pos = signal({ x: 0, y: 0 });
subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
  $mouse_pos.value = { x, y };
});

export const $mouse_at_edge = computed<FancyToolbarSide | null>(() => {
  if ($mouse_pos.value.y === $current_monitor.value.rect.top) {
    return FancyToolbarSide.Top;
  }
  if ($mouse_pos.value.y === $current_monitor.value.rect.bottom - 1) {
    return FancyToolbarSide.Bottom;
  }
  return null;
});
