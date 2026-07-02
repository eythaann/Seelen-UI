import { Widget } from "@seelen-ui/lib";
import type { PhysicalMonitor } from "@seelen-ui/lib/types";
import { globalState } from "./mod.svelte";
import { StartDisplayMode, StartView } from "../constants";

let monitorToShow = $derived.by(() => {
  let targetMonitor: PhysicalMonitor | undefined;

  if (globalState.desiredMonitorId) {
    targetMonitor = globalState.monitors.find((m) => m.id === globalState.desiredMonitorId);
  }

  // No explicit monitor: use the one under the requested point (mouse cursor).
  if (!targetMonitor && globalState.desiredPosition) {
    const p = globalState.desiredPosition;
    targetMonitor = globalState.monitors.find((m) =>
      p.x >= m.rect.left && p.x < m.rect.right && p.y >= m.rect.top && p.y < m.rect.bottom
    );
  }

  // Fallback to primary monitor if not found or not specified
  if (!targetMonitor) {
    targetMonitor = globalState.monitors.find((m) => m.isPrimary) || globalState.monitors[0];
  }

  return targetMonitor;
});

async function placeCenteredToMonitor(targetMonitor: PhysicalMonitor): Promise<void> {
  const widget = Widget.getCurrent();
  const monitorWidth = targetMonitor.rect.right - targetMonitor.rect.left;
  const monitorHeight = targetMonitor.rect.bottom - targetMonitor.rect.top;

  // globalState.displayMode === StartDisplayMode.Fullscreen
  let x = targetMonitor.rect.left;
  let y = targetMonitor.rect.top;
  let width = monitorWidth;
  let height = monitorHeight;

  if (globalState.displayMode === StartDisplayMode.Normal) {
    width = Math.round(Math.min(monitorWidth * 0.6, 1200 * targetMonitor.scaleFactor));
    height = Math.round(Math.min(monitorHeight * 0.6, 1200 * targetMonitor.scaleFactor));

    const monitorCenterX = targetMonitor.rect.left + monitorWidth / 2;
    const monitorCenterY = targetMonitor.rect.top + monitorHeight / 2;

    x = Math.round(monitorCenterX - width / 2);
    y = Math.round(monitorCenterY - height / 2);
  }

  await widget.setPosition({
    left: x,
    top: y,
    right: x + width,
    bottom: y + height,
  });
}

$effect.root(() => {
  $effect(() => {
    globalState.displayMode;
    if (monitorToShow) {
      placeCenteredToMonitor(monitorToShow);
    }
  });
});

export async function onTriggered(
  monitorId?: string | null,
  position?: { x: number; y: number } | null,
) {
  globalState.view = StartView.Favorites;
  globalState.desiredMonitorId = monitorId || null;
  globalState.desiredPosition = position || null;
  globalState.version++; // trigger reactive updates

  await Widget.self.show();
  await Widget.self.focus();
}
