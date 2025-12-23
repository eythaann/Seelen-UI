import { computed } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide } from "@seelen-ui/lib/types";
import { lazySignal } from "libs/ui/react/utils/LazySignal";

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

export const $virtual_desktop = lazySignal(async () => {
  const initialDesktops = await invoke(SeelenCommand.StateGetVirtualDesktops);
  return initialDesktops.monitors[currentMonitorId];
});
await subscribe(SeelenEvent.VirtualDesktopsChanged, (e) => {
  $virtual_desktop.value = e.payload.monitors[currentMonitorId];
});
await $virtual_desktop.init();

export const $monitors = lazySignal(() => invoke(SeelenCommand.SystemGetMonitors));
await subscribe(SeelenEvent.SystemMonitorsChanged, $monitors.setByPayload);
await $monitors.init();

export const $current_monitor = computed(
  () => $monitors.value.find((m) => m.id === currentMonitorId)!,
);

const $mouse_pos = lazySignal(() => invoke(SeelenCommand.GetMousePosition));
await subscribe(SeelenEvent.GlobalMouseMove, $mouse_pos.setByPayload);
await $mouse_pos.init();

export const $mouse_at_edge = computed<FancyToolbarSide | null>(() => {
  const box = $current_monitor.value.rect;
  const x = $mouse_pos.value[0];
  const y = $mouse_pos.value[1];

  if (x < box.left || x > box.right || y < box.top || y > box.bottom) {
    return null;
  }

  if (y === box.top) {
    return FancyToolbarSide.Top;
  }

  if (y === box.bottom - 1) {
    return FancyToolbarSide.Bottom;
  }
  return null;
});
