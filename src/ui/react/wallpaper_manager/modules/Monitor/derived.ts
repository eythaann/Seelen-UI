import { computed } from "@preact/signals";

import { $monitors } from "../shared/state.ts";

export const $desktopRect = computed(() => {
  let rect = { top: 0, left: 0, right: 0, bottom: 0 };
  for (const monitor of $monitors.value) {
    rect.left = Math.min(rect.left, monitor.rect.left);
    rect.top = Math.min(rect.top, monitor.rect.top);
    rect.right = Math.max(rect.right, monitor.rect.right);
    rect.bottom = Math.max(rect.bottom, monitor.rect.bottom);
  }
  return rect;
});

export const $relativeMonitors = computed(() => {
  return $monitors.value.map((monitor) => {
    return {
      ...monitor,
      rect: {
        ...monitor.rect,
        left: monitor.rect.left - $desktopRect.value.left,
        top: monitor.rect.top - $desktopRect.value.top,
        right: monitor.rect.right - $desktopRect.value.left,
        bottom: monitor.rect.bottom - $desktopRect.value.top,
      },
    };
  });
});
