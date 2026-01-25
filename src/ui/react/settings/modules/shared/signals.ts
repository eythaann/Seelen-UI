import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { signal } from "@preact/signals";

export const $virtual_desktops = lazySignal(async () => {
  return await invoke(SeelenCommand.StateGetVirtualDesktops);
});

await subscribe(SeelenEvent.VirtualDesktopsChanged, (event) => {
  $virtual_desktops.value = event.payload;
});

await $virtual_desktops.init();

/**
 * Session-only set of wallpaper IDs that failed thumbnail extraction
 * These wallpapers will be marked as corrupted and won't be retried
 */
export const $corruptedWallpapers = signal<Set<string>>(new Set());
