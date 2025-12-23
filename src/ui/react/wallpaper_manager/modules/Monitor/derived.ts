import { computed } from "@preact/signals";

import { $monitors } from "../shared/state.ts";

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
