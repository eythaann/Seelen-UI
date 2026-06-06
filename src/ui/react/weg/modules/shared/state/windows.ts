import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import type { FocusedApp, UserAppWindow } from "@seelen-ui/lib/types";
import { $widget_rect } from "./settings";
import { computed, signal } from "@preact/signals";
import { debounce } from "lodash";
import type { AppOrFileWegItem } from "../types";

const widget = Widget.getCurrent();
const selfWinId = await invoke(SeelenCommand.GetSelfWindowId);

export const $interactables = lazySignal(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, $interactables.setByPayload);

export const $top_interactable_window = computed(() =>
  $interactables.value.find((w) => w.monitor === widget.decoded.monitorId && !w.isIconic)
);

export const $previews = lazySignal(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, $previews.setByPayload);

/** Used to check which window was last focused on interactions with the current window */
export const $delayedFocused = signal<FocusedApp | null>(null);
export const $focused = lazySignal(() => invoke(SeelenCommand.GetFocusedApp));

const setDelayedFocused = debounce((v: FocusedApp) => {
  $delayedFocused.value = v;
}, 200);

subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  $focused.value = e.payload;

  setDelayedFocused(e.payload);
  if (e.payload.hwnd !== selfWinId) {
    setDelayedFocused.flush();
  }
});

export const $widget_statuses = lazySignal(() => invoke(SeelenCommand.DebugGetWidgetsStatuses));
subscribe(SeelenEvent.WidgetDebugInfoChanged, $widget_statuses.setByPayload);

await Promise.all([
  $interactables.init(),
  $previews.init(),
  $focused.init(),
  $widget_statuses.init(),
]);

export const $is_dock_overlapped = computed(() => {
  const focused = $focused.value;
  const by = focused?.monitor === widget.decoded.monitorId ? focused : null;
  const interactables = $interactables.value;

  if (!by || !by.rect) {
    return false;
  }

  if (!interactables.some((w) => w.hwnd === by.hwnd)) {
    return false;
  }

  const a = $widget_rect.value.hitboxRect;
  const b = by.rect;

  // The edge pixel overlapping do not matters. This resolves the shared pixel in between the monitors,
  // hereby a fullscreened app shared pixel collision does not hide other monitor windows.
  if (a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom) {
    return false;
  }

  return true;
});

// on change of this function update src\background\widgets\weg\cli.rs too.
//
// Grouping rules:
//   1. Window has a umid  → matched only by exact umid equality. Path is not used.
//      If no item has that umid, a new item will be created for it.
//   2. Window has no umid → matched by exact path (item.relaunch.command or item.path).
export function getWindowsForItem(
  item: AppOrFileWegItem,
  interactables: UserAppWindow[],
): UserAppWindow[] {
  const itemCommand = item.relaunch?.command.toLowerCase();
  const itemPath = item.path.toLowerCase();

  return interactables.filter((w) => {
    if (w.umid) {
      // Rule 1: window carries a umid — only an item with the exact same umid may claim it.
      return item.umid === w.umid;
    }
    // Rule 2: window has no umid — match by path.
    const winPath = w.process.path?.toLowerCase() ?? "";
    return winPath !== "" && (itemCommand === winPath || itemPath === winPath);
  });
}
