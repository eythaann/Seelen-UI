import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

export const virtualDesktop = lazyRune(async () => {
  const initialDesktops = await invoke(SeelenCommand.StateGetVirtualDesktops);
  return initialDesktops.monitors[currentMonitorId];
});
subscribe(SeelenEvent.VirtualDesktopsChanged, (e) => {
  virtualDesktop.value = e.payload.monitors[currentMonitorId];
});
await virtualDesktop.init();

const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

const _currentMonitor = $derived(monitors.value.find((m) => m.id === currentMonitorId)!);

const mousePos = lazyRune(() => invoke(SeelenCommand.GetMousePosition));
subscribe(SeelenEvent.GlobalMouseMove, mousePos.setByPayload);
await mousePos.init();

const _mouseAtEdge = $derived.by((): FancyToolbarSide | null => {
  const box = _currentMonitor.rect;
  const x = mousePos.value[0];
  const y = mousePos.value[1];

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

class SystemState {
  get currentMonitor() {
    return _currentMonitor;
  }

  get mouseAtEdge(): FancyToolbarSide | null {
    return _mouseAtEdge;
  }
}

export const systemState = new SystemState();
