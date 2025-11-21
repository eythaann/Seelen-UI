import { computed } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide } from "@seelen-ui/lib/types";
import { lazySignal } from "@shared/LazySignal";

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
  if ($mouse_pos.value[1] === $current_monitor.value.rect.top) {
    return FancyToolbarSide.Top;
  }
  if ($mouse_pos.value[1] === $current_monitor.value.rect.bottom - 1) {
    return FancyToolbarSide.Bottom;
  }
  return null;
});
