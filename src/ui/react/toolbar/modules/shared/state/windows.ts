import { computed, effect, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { type FocusedApp, HideMode } from "@seelen-ui/lib/types";
import { debounce } from "lodash";
import { $settings, $widget_rect } from "./mod";
import { $mouse_at_edge } from "./system";
import { $is_this_webview_focused } from "libs/ui/react/utils/signals";
import { $there_are_open_popups } from "@shared/components/AnimatedWrappers/PopupsState";
import { lazySignal } from "libs/ui/react/utils/LazySignal";

const widget = Widget.getCurrent();

export const $interactables = lazySignal(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, $interactables.setByPayload);
await $interactables.init();

export const $thereIsMaximizedOnBg = computed(() => {
  return $interactables.value.some(
    (w) => !w.isIconic && w.isZoomed && w.monitor === widget.decoded.monitorId,
  );
});

export const $focused = lazySignal(() => invoke(SeelenCommand.GetFocusedApp));
export const $lastFocusedOnMonitor = lazySignal<FocusedApp | null>(async () => {
  const focused = await invoke(SeelenCommand.GetFocusedApp);
  return focused.monitor === widget.decoded.monitorId ? focused : null;
});
await subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  $focused.value = e.payload;
  if (e.payload.monitor === widget.decoded.monitorId) {
    $lastFocusedOnMonitor.value = e.payload;
  }
});
await $focused.init();
await $lastFocusedOnMonitor.init();

export const $is_tb_overlapped = computed(() => {
  const by = $lastFocusedOnMonitor.value;
  const interactables = $interactables.value;

  if (!by || !by.rect) {
    return false;
  }

  if (!interactables.some((w) => w.hwnd === by.hwnd)) {
    return false;
  }

  const a = $widget_rect.value;
  const b = by.rect;

  // The edge pixel overlapping do not matters. This resolves the shared pixel in between the monitors,
  // hereby a fullscreened app shared pixel collision does not hide other monitor windows.
  if (a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom) {
    return false;
  }

  return true;
});

export const $hidden_by_autohide = signal(false);
const setToolbarAsHidden = computed(() => {
  return debounce(() => ($hidden_by_autohide.value = true), $settings.value.delayToHide);
});
const setToolbarAsNotHidden = computed(() => {
  return debounce(() => ($hidden_by_autohide.value = false), $settings.value.delayToShow);
});

effect(() => {
  Widget.self.webview.setIgnoreCursorEvents($hidden_by_autohide.value);
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
      hidden = !$is_this_webview_focused.value && !$there_are_open_popups.value && !isMouseOverEdge;
      break;
    case HideMode.OnOverlap:
      hidden = $is_tb_overlapped.value &&
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
