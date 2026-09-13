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

const _mouseAtEdge = $derived.by((): FancyToolbarSide | null => {
  const box = _currentMonitor.rect;
  const x = mousePos.value[0];
  const y = mousePos.value[1];

  const threshold = Math.max(3, Math.round(2 * (_currentMonitor.scaleFactor || 1)));

  if (x < box.left - 1 || x > box.right + 1 || y < box.top - 1 || y > box.bottom + 1) {
    return null;
  }

  if (y <= box.top + threshold) {
    return FancyToolbarSide.Top;
  }

  if (y >= box.bottom - threshold) {
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
