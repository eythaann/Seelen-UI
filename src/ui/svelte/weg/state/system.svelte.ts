import { SeelenWegSide } from "@seelen-ui/lib/types";
import { currentMonitorId, monitors, mousePos } from "./getters.svelte.ts";

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

const _mouseAtEdge = $derived.by((): SeelenWegSide | null => {
  const box = _currentMonitor.rect;
  const x = mousePos.value.x;
  const y = mousePos.value.y;

  const isOutHorizontally = x < box.left || x > box.right;
  if (!isOutHorizontally) {
    if (inRange(y, box.top)) return SeelenWegSide.Top;
    if (inRange(y, box.bottom - 1)) return SeelenWegSide.Bottom;
  }

  const isOutVertically = y < box.top || y > box.bottom;
  if (!isOutVertically) {
    if (inRange(x, box.left)) return SeelenWegSide.Left;
    if (inRange(x, box.right - 1)) return SeelenWegSide.Right;
  }

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
