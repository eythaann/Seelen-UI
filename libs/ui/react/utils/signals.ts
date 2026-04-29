import { SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { signal } from "@preact/signals";

export const $is_this_webview_focused = signal(false);
subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: { hwnd, ownerHwnd } }) => {
  $is_this_webview_focused.value = Widget.self.windowId === hwnd || Widget.self.windowId === ownerHwnd;
});
