import { effect, signal } from "@preact/signals";
import { HideMode } from "@seelen-ui/lib/types";
import { $is_this_webview_focused, $is_touch_primary } from "libs/ui/react/utils/signals.ts";

import { $mouse_at_edge } from "./system.ts";
import { $settings } from "./settings.ts";
import { $is_dock_overlapped } from "./windows.ts";

export const $dock_should_be_hidden = signal(false);

effect(() => {
  const { delayToHide, delayToShow, hideMode, position } = $settings.value;

  let hidden = false;
  let flush = false;

  const isMouseOverEdge = $mouse_at_edge.value === position;

  switch (hideMode) {
    case HideMode.Never:
      hidden = false;
      flush = true;
      break;
    case HideMode.Always:
      hidden = !$is_touch_primary.value && !$is_this_webview_focused.value && !isMouseOverEdge;
      flush = flush = $is_touch_primary.value;
      break;
    case HideMode.OnOverlap:
      hidden = !$is_touch_primary.value &&
        $is_dock_overlapped.value &&
        !$is_this_webview_focused.value &&
        !isMouseOverEdge;
      flush = $is_touch_primary.value;
      break;
  }

  let timeout: ReturnType<typeof setTimeout> | null = null;
  if (hidden) {
    timeout = setTimeout(() => ($dock_should_be_hidden.value = true), delayToHide);
  } else {
    if (flush) {
      $dock_should_be_hidden.value = false;
    } else {
      timeout = setTimeout(() => ($dock_should_be_hidden.value = false), delayToShow);
    }
  }

  return () => {
    if (timeout) {
      clearTimeout(timeout);
    }
  };
});
