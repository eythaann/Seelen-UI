import { FancyToolbarSide } from "@seelen-ui/lib/types";
import { currentMonitorId, monitors, mousePos, virtualDesktop } from "./getters.svelte.ts";

export { virtualDesktop };

const _currentMonitor = $derived.by(() => {
  const monitor = monitors.value.find((m) => m.id === currentMonitorId);
  if (!monitor) {
    throw new Error("Current monitor not found");
  }
  return monitor;
});

const THRESHOLD = 2;
function inRange(value: number, origin: number) {
  return value >= origin - THRESHOLD && value <= origin + THRESHOLD;
}

const _mouseAtEdge = $derived.by((): FancyToolbarSide | null => {
  const box = _currentMonitor.rect;
  const x = mousePos.value[0];
  const y = mousePos.value[1];

  const isOutHorizontally = x < box.left || x > box.right;
  if (!isOutHorizontally) {
    if (inRange(y, box.top)) return FancyToolbarSide.Top;
    if (inRange(y, box.bottom - 1)) return FancyToolbarSide.Bottom;
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
