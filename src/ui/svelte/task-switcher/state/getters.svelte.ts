import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";
import { monitors } from "libs/ui/svelte/runes/monitors.svelte.ts";
import { PreviewSnapshots } from "libs/ui/svelte/utils/PreviewSnapshots.svelte.ts";

export const widget = Widget.getCurrent();

export const settings = lazyRune(() => Settings.getAsync());
await Settings.onChange((s) => (settings.value = s));

export const windows = lazyRune(async () =>
  (await invoke(SeelenCommand.GetUserAppWindows)).toSorted(
    (a, b) => b.lastForegroundAt - a.lastForegroundAt,
  )
);
subscribe(SeelenEvent.UserAppWindowsChanged, ({ payload }) => {
  windows.value = payload.toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt);
});

export const previews = new PreviewSnapshots();
widget.onVisibilityChange((visible) => {
  if (visible) previews.start();
  else previews.stop();
});

export const focusedWinId = lazyRune(async () => (await invoke(SeelenCommand.GetFocusedApp)).id);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedWinId.value = e.payload.id;
});

await Promise.all([
  settings.init(),
  windows.init(),
  focusedWinId.init(),
  monitors.init(),
]);
