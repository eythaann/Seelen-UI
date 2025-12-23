import { UIColors } from "@seelen-ui/lib";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { lazySignal } from "./LazySignal";

const webview = getCurrentWindow();
export const $is_this_webview_focused = lazySignal(() => webview.isFocused());
await webview.onFocusChanged(async () => {
  // the payload value is not used, cuz on startup it gives wrong value.
  $is_this_webview_focused.value = await webview.isFocused();
});
await $is_this_webview_focused.init();

export const $system_colors = lazySignal(async () => (await UIColors.getAsync()).inner);
await UIColors.onChange((colors) => ($system_colors.value = colors.inner));
await $system_colors.init();
