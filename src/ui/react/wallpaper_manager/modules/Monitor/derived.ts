import { computed } from "@preact/signals";
import { SeelenWallWidgetId } from "@seelen-ui/lib";
import type { Wallpaper, WallpaperId } from "@seelen-ui/lib/types";

import { $monitors, $settings, $wallpapers } from "../shared/state.ts";

export const $relativeMonitors = computed(() => {
  const lower = { x: 0, y: 0 };
  for (const monitor of $monitors.value) {
    if (monitor.rect.left < lower.x) {
      lower.x = monitor.rect.left;
    }
    if (monitor.rect.top < lower.y) {
      lower.y = monitor.rect.top;
    }
  }

  return $monitors.value.map((monitor) => {
    return {
      ...monitor,
      rect: {
        ...monitor.rect,
        left: monitor.rect.left - lower.x,
        top: monitor.rect.top - lower.y,
        right: monitor.rect.right - lower.x,
        bottom: monitor.rect.bottom - lower.y,
      },
    };
  });
});

// active wallpapers, sorted by user settings and skip missing/removed wallpapers
export function $get_active_wallpapers(monitorId: string): Wallpaper[] {
  const active: Wallpaper[] = [];

  const monitorConfig = $settings.value.byMonitor[monitorId];
  const bgsInMonitor = monitorConfig?.byWidget?.[SeelenWallWidgetId]
    ?.backgroundsV2 as
      | WallpaperId[]
      | undefined;

  for (const id of bgsInMonitor || $settings.value.backgroundsV2) {
    let current = $wallpapers.value.find((w) => w.id === id);
    // if doesn't exist, active wallpaper was removed
    if (current) {
      active.push(current);
    }
  }

  return active;
}
