import { Widget } from "@seelen-ui/lib";
import type { PhysicalMonitor } from "@seelen-ui/lib/types";
import { globalState } from "./mod.svelte";
import { StartDisplayMode, StartView } from "../constants";

let desiredPosition = $state<{ x: number; y: number } | null>(null);

// Monitor under the cursor position that triggered the menu, falling back to primary
let monitorToShow = $derived.by(() => {
  const pos = desiredPosition;
  let targetMonitor: PhysicalMonitor | undefined;

  if (pos) {
    targetMonitor = globalState.monitors.find(
      (m) =>
        m.rect.left <= pos.x &&
        pos.x < m.rect.right &&
        m.rect.top <= pos.y &&
        pos.y < m.rect.bottom,
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
    width = Math.min(monitorWidth * 0.55, 1200 * targetMonitor.scaleFactor);
    height = Math.min(monitorHeight * 0.55, 1200 * targetMonitor.scaleFactor);

    const monitorCenterX = targetMonitor.rect.left + monitorWidth / 2;
    const monitorCenterY = targetMonitor.rect.top + monitorHeight / 2;

    x = monitorCenterX - width / 2;
    y = monitorCenterY - height / 2;
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

export async function onTriggered(cursorPosition?: { x: number; y: number } | null) {
  desiredPosition = cursorPosition ?? null;

  globalState.view = StartView.Favorites;
  globalState.version++; // trigger reactive updates

  await Widget.self.show();
  await Widget.self.focus();
}
