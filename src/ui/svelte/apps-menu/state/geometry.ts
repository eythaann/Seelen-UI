import type { PhysicalMonitor, Rect } from "@seelen-ui/lib/types";
import { StartDisplayMode } from "../constants";

export function resolveTargetMonitor(
  monitors: PhysicalMonitor[],
  monitorId?: string | null,
): PhysicalMonitor | undefined {
  return (
    monitors.find((monitor) => monitor.id === monitorId) ||
    monitors.find((monitor) => monitor.isPrimary) ||
    monitors[0]
  );
}

export function getAppsMenuRect(
  targetMonitor: PhysicalMonitor,
  displayMode: StartDisplayMode,
): Rect {
  const monitorWidth = targetMonitor.rect.right - targetMonitor.rect.left;
  const monitorHeight = targetMonitor.rect.bottom - targetMonitor.rect.top;

  if (displayMode === StartDisplayMode.Fullscreen) {
    return { ...targetMonitor.rect };
  }

  const width = Math.round(Math.min(monitorWidth * 0.6, 1200 * targetMonitor.scaleFactor));
  const height = Math.round(Math.min(monitorHeight * 0.6, 1200 * targetMonitor.scaleFactor));
  const monitorCenterX = targetMonitor.rect.left + monitorWidth / 2;
  const monitorCenterY = targetMonitor.rect.top + monitorHeight / 2;
  const left = Math.round(monitorCenterX - width / 2);
  const top = Math.round(monitorCenterY - height / 2);

  return {
    left,
    top,
    right: left + width,
    bottom: top + height,
  };
}
