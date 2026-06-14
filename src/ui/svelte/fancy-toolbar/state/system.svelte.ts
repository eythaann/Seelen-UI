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

export const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

export const currentMonitor = {
  get value() {
    return monitors.value.find((m) => m.id === currentMonitorId)!;
  },
};

const mousePos = lazyRune(() => invoke(SeelenCommand.GetMousePosition));
subscribe(SeelenEvent.GlobalMouseMove, mousePos.setByPayload);
await mousePos.init();

export const mouseAtEdge = {
  get value(): FancyToolbarSide | null {
    const box = currentMonitor.value.rect;
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
  },
};
