import { UIColors, Widget } from "@seelen-ui/lib";
import { lazySignal } from "./LazySignal";

const window = Widget.self.window;

export const $is_this_webview_focused = lazySignal(() => window.isFocused());
await window.onFocusChanged(async () => {
  // the payload value is not used, cuz on startup it gives wrong value.
  $is_this_webview_focused.value = await window.isFocused();
});
await $is_this_webview_focused.init();

export const $system_colors = lazySignal(async () => (await UIColors.getAsync()).inner);
await UIColors.onChange((colors) => ($system_colors.value = colors.inner));
await $system_colors.init();
