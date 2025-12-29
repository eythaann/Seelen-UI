import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { Widget } from "@seelen-ui/lib";
import type { PhysicalMonitor } from "@seelen-ui/lib/types";
import { globalState } from "./state.svelte";
import { StartDisplayMode, StartView } from "./constants";

let desiredMonitorId = $state<string | null>();
let showAfterPositioning = $state({ value: false });

let monitorToShow = $derived.by(() => {
  let targetMonitor: PhysicalMonitor | undefined;

  if (desiredMonitorId) {
    targetMonitor = globalState.monitors.find((m) => m.id === desiredMonitorId);
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
    width = Math.min(monitorWidth * 0.6, 1200 * targetMonitor.scaleFactor);
    height = Math.min(monitorHeight * 0.6, 1200 * targetMonitor.scaleFactor);

    const monitorCenterX = targetMonitor.rect.left + monitorWidth / 2;
    const monitorCenterY = targetMonitor.rect.top + monitorHeight / 2;

    x = Math.round(monitorCenterX - width / 2);
    y = Math.round(monitorCenterY - height / 2);
  }

  console.debug("WTF???", globalState.displayMode, monitorWidth, monitorHeight, width, height);

  await widget.webview.setShadow(globalState.displayMode === StartDisplayMode.Normal);
  await widget.webview.setPosition(new PhysicalPosition(x, y));
  await widget.webview.setSize(new PhysicalSize({ width, height }));

  if (showAfterPositioning.value) {
    await widget.webview.show();
  }
}

$effect.root(() => {
  $effect(() => {
    globalState.displayMode;
    showAfterPositioning.value;

    if (monitorToShow && showAfterPositioning.value) {
      placeCenteredToMonitor(monitorToShow);
    }
  });
});

export function showTriggered(monitorId?: string | null) {
  globalState.view = StartView.Favorites;
  desiredMonitorId = monitorId;
  showAfterPositioning.value = true;
}

export function hideTriggered() {
  showAfterPositioning.value = false;
  Widget.getCurrent().webview.hide();
}
