import { signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SysTrayIcon } from "@seelen-ui/lib/types";

/**
 * Live system tray icons. Pinned tray toolbar items render their icon from this
 * signal (by id) instead of baking the icon into the item template, so the icon
 * updates live without rewriting/persisting the toolbar state on every change.
 */
export const $tray_icons = signal<SysTrayIcon[]>([]);

invoke(SeelenCommand.GetSystemTrayIcons).then((items) => {
  $tray_icons.value = items;
}).catch(() => {});

subscribe(SeelenEvent.SystemTrayChanged, ({ payload }) => {
  $tray_icons.value = payload;
});
