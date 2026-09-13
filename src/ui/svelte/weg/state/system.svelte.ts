import { SeelenWegSide } from "@seelen-ui/lib/types";
import { currentMonitorId, monitors, mousePos } from "./getters.svelte.ts";

const _currentMonitor = $derived.by(() => {
  const monitor = monitors.value.find((m) => m.id === currentMonitorId);
  if (!monitor) {
    throw new Error("Current monitor not found");
  }
  return monitor;
});

const _mouseAtEdge = $derived.by((): SeelenWegSide | null => {
  const box = _currentMonitor.rect;
  const x = mousePos.value.x;
  const y = mousePos.value.y;

  const threshold = Math.max(3, Math.round(2 * (_currentMonitor.scaleFactor || 1)));

  if (x < box.left - 1 || x > box.right + 1 || y < box.top - 1 || y > box.bottom + 1) {
    return null;
  }

  if (y <= box.top + threshold) return SeelenWegSide.Top;
  if (x <= box.left + threshold) return SeelenWegSide.Left;
  if (y >= box.bottom - threshold) return SeelenWegSide.Bottom;
  if (x >= box.right - threshold) return SeelenWegSide.Right;

  return null;
});

class SystemState {
  get currentMonitor() {
    return _currentMonitor;
  }

  get mouseAtEdge(): SeelenWegSide | null {
    return _mouseAtEdge;
  }
}

export const systemState = new SystemState();
