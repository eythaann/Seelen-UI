import { computed, effect, signal } from "@preact/signals";
import { SeelenEvent, Settings } from "@seelen-ui/lib";
import { HideMode, type SeelenWegSettings } from "@seelen-ui/lib/types";
import { $is_this_webview_focused } from "@shared/signals";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { debounce } from "lodash";

import { $mouse_at_edge } from "./system.ts";

export const $settings = signal<SeelenWegSettings>(
  (await Settings.getAsync()).seelenweg,
);
Settings.onChange((settings) => ($settings.value = settings.seelenweg));

export const $is_dock_overlaped = signal(false);
await getCurrentWebviewWindow().listen<boolean>(
  SeelenEvent.WegOverlaped,
  (event) => {
    $is_dock_overlaped.value = event.payload;
  },
);

export const $open_popups = signal<Record<string, boolean>>({});
export const $there_are_open_popups = computed(() => Object.values($open_popups.value).some((v) => v));

export const $dock_should_be_hidden = signal(false);
const setDockAsHidden = computed(() => {
  return debounce(
    () => ($dock_should_be_hidden.value = true),
    $settings.value.delayToHide,
  );
});
const setDockAsNotHidden = computed(() => {
  return debounce(
    () => ($dock_should_be_hidden.value = false),
    $settings.value.delayToShow,
  );
});

effect(() => {
  let hidden = false;
  let flush = false;

  let isMouseOverEdge = $mouse_at_edge.value === $settings.value.position;

  switch ($settings.value.hideMode) {
    case HideMode.Never:
      hidden = false;
      flush = true;
      break;
    case HideMode.Always:
      hidden = !$is_this_webview_focused.value &&
        !$there_are_open_popups.value && !isMouseOverEdge;
      break;
    case HideMode.OnOverlap:
      hidden = $is_dock_overlaped.value &&
        !$is_this_webview_focused.value &&
        !$there_are_open_popups.value &&
        !isMouseOverEdge;
      break;
  }

  if (hidden) {
    setDockAsNotHidden.peek().cancel();
    setDockAsHidden.peek()();
    return;
  }

  setDockAsHidden.peek().cancel();
  setDockAsNotHidden.peek()();
  if (flush) {
    setDockAsNotHidden.peek().flush();
  }
});
