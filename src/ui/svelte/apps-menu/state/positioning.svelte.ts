import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { Widget } from "@seelen-ui/lib";
import type { PhysicalMonitor } from "@seelen-ui/lib/types";
import { globalState } from "./mod.svelte";
import { StartDisplayMode, StartView } from "../constants";

let monitorToShow = $derived.by(() => {
  let targetMonitor: PhysicalMonitor | undefined;

  if (globalState.desiredMonitorId) {
    targetMonitor = globalState.monitors.find((m) => m.id === globalState.desiredMonitorId);
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

  await widget.webview.setShadow(globalState.displayMode === StartDisplayMode.Normal);
  await widget.webview.setPosition(new PhysicalPosition(x, y));
  await widget.webview.setSize(new PhysicalSize({ width, height }));

  if (globalState.showing) {
    await widget.show();
    await widget.focus();
  }
}

$effect.root(() => {
  $effect(() => {
    globalState.displayMode;
    if (monitorToShow && globalState.showing) {
      placeCenteredToMonitor(monitorToShow);
    }
  });

  $effect(() => {
    if (!globalState.showing) {
      Widget.self.hide();
    }
  });
});

export function onTriggered(monitorId?: string | null) {
  globalState.view = StartView.Favorites;
  globalState.desiredMonitorId = monitorId || null;
  globalState.showing = true;
}
