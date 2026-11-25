import { SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { signal } from "@preact/signals";

export const $is_this_webview_focused = signal(false);
subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: { id, ownerId } }) => {
  $is_this_webview_focused.value = Widget.self.windowId === id || Widget.self.windowId === ownerId;
});

const _pointerQuery = window.matchMedia("(hover: hover) and (pointer: fine)");
export const $is_touch_primary = signal(!_pointerQuery.matches);
_pointerQuery.addEventListener("change", (e) => {
  $is_touch_primary.value = !e.matches;
});
