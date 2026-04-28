import { SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { lazySignal } from "./LazySignal";

export const $is_this_webview_focused = lazySignal(() => Widget.self.window.isFocused());
await subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: { hwnd, ownerHwnd } }) => {
  $is_this_webview_focused.value = Widget.self.windowId === hwnd || Widget.self.windowId === ownerHwnd;
});
await $is_this_webview_focused.init();
