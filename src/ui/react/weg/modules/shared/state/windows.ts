import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import type { FocusedApp } from "@seelen-ui/lib/types";
import { $widget_rect } from "./settings";
import { computed, signal } from "@preact/signals";
import { debounce } from "lodash";

const widget = Widget.getCurrent();
const selfWinId = await invoke(SeelenCommand.GetSelfWindowId);

export const $interactables = lazySignal(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, $interactables.setByPayload);
await $interactables.init();

export const $previews = lazySignal(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
await subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, $previews.setByPayload);
await $previews.init();

/** Used to check which window was last focused on interactions with the current window */
export const $delayedFocused = signal<FocusedApp | null>(null);

export const $focused = lazySignal(() => invoke(SeelenCommand.GetFocusedApp));
export const $lastFocusedOnMonitor = lazySignal<FocusedApp | null>(async () => {
  const focused = await invoke(SeelenCommand.GetFocusedApp);
  return focused.monitor === widget.decoded.monitorId ? focused : null;
});

const setDelayedFocused = debounce((v: FocusedApp) => {
  $delayedFocused.value = v;
}, 200);

await subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  $focused.value = e.payload;

  if (e.payload.monitor === widget.decoded.monitorId) {
    $lastFocusedOnMonitor.value = e.payload;
  }

  setDelayedFocused(e.payload);
  if (e.payload.hwnd !== selfWinId) {
    setDelayedFocused.flush();
  }
});
await $focused.init();
await $lastFocusedOnMonitor.init();

export const $is_dock_overlapped = computed(() => {
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
