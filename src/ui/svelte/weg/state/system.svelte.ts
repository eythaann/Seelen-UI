import { SeelenWegSide } from "@seelen-ui/lib/types";
import { currentMonitorId, monitors, mousePos, notifications, players, trashBinInfo } from "./getters.svelte.ts";

export { notifications, players, trashBinInfo };

const _currentMonitor = $derived(
  monitors.value.find((m) => m.id === currentMonitorId) ??
    monitors.value.find((m) => m.isPrimary) ??
    monitors.value[0]!,
);

const _mouseAtEdge = $derived.by((): SeelenWegSide | null => {
  const box = _currentMonitor.rect;
  const x = mousePos.value.x;
  const y = mousePos.value.y;

  if (x < box.left || x > box.right || y < box.top || y > box.bottom) {
    return null;
  }

  if (y === box.top) return SeelenWegSide.Top;
  if (x === box.left) return SeelenWegSide.Left;
  if (y === box.bottom - 1) return SeelenWegSide.Bottom;
  if (x === box.right - 1) return SeelenWegSide.Right;

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
