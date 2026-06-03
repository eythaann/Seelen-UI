import { effect, signal } from "@preact/signals";
import { HideMode } from "@seelen-ui/lib/types";
import { $settings } from "./settings";
import { $mouse_at_edge } from "./system";
import { $is_tb_overlapped } from "./windows";
import { $is_this_webview_focused, $is_touch_primary } from "libs/ui/react/utils/signals";

export const $hidden_by_autohide = signal(false);

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
      flush = $is_touch_primary.value;
      break;
    case HideMode.OnOverlap:
      hidden = !$is_touch_primary.value &&
        $is_tb_overlapped.value &&
        !$is_this_webview_focused.value &&
        !isMouseOverEdge;
      flush = $is_touch_primary.value;
      break;
  }

  let timeout: ReturnType<typeof setTimeout> | null = null;
  if (hidden) {
    timeout = setTimeout(() => ($hidden_by_autohide.value = true), delayToHide);
  } else {
    if (flush) {
      $hidden_by_autohide.value = false;
    } else {
      timeout = setTimeout(() => ($hidden_by_autohide.value = false), delayToShow);
    }
  }

  return () => {
    if (timeout) {
      clearTimeout(timeout);
    }
  };
});
