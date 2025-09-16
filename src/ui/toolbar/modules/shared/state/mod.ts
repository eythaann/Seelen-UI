import { computed, effect, signal } from "@preact/signals";
import { HideMode, SeelenEvent, Settings } from "@seelen-ui/lib";
import { FancyToolbarSettings } from "@seelen-ui/lib/types";
import { $there_are_open_popups } from "@shared/components/AnimatedWrappers/PopupsState";
import { $is_this_webview_focused } from "@shared/signals";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { debounce } from "lodash";

import { $mouse_at_edge } from "./system";

const initialSettings = await Settings.getAsync();
export const $settings = signal<
  FancyToolbarSettings & Pick<Settings, "dateFormat">
>({
  ...initialSettings.fancyToolbar,
  dateFormat: initialSettings.dateFormat,
});
Settings.onChange(
  (settings) => ($settings.value = {
    ...settings.fancyToolbar,
    dateFormat: settings.dateFormat,
  }),
);

export const $is_toolbar_overlaped = signal(false);
await getCurrentWebviewWindow().listen<boolean>(
  SeelenEvent.ToolbarOverlaped,
  (event) => {
    $is_toolbar_overlaped.value = event.payload;
  },
);

export const $bar_should_be_hidden = signal(false);
const setToolbarAsHidden = computed(() => {
  return debounce(
    () => ($bar_should_be_hidden.value = true),
    $settings.value.delayToHide,
  );
});
const setToolbarAsNotHidden = computed(() => {
  return debounce(
    () => ($bar_should_be_hidden.value = false),
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
      hidden = $is_toolbar_overlaped.value &&
        !$is_this_webview_focused.value &&
        !$there_are_open_popups.value &&
        !isMouseOverEdge;
      break;
  }

  if (hidden) {
    setToolbarAsNotHidden.peek().cancel();
    setToolbarAsHidden.peek()();
    return;
  }

  setToolbarAsHidden.peek().cancel();
  setToolbarAsNotHidden.peek()();
  if (flush) {
    setToolbarAsNotHidden.peek().flush();
  }
});
