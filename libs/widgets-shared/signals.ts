import { signal } from "@preact/signals";
import { UIColors } from "@seelen-ui/lib";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const $is_this_webview_focused = signal(
  await getCurrentWindow().isFocused(),
);
getCurrentWindow().onFocusChanged((event) => {
  $is_this_webview_focused.value = event.payload;
});

export const $system_colors = signal((await UIColors.getAsync()).inner);
UIColors.onChange((colors) => ($system_colors.value = colors.inner));
