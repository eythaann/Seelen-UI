import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SysTrayIcon } from "@seelen-ui/lib/types";
import { persistentRune } from "libs/ui/svelte/utils";

export const state = $state({
  trayItems: await invoke(SeelenCommand.GetSystemTrayIcons),
});

subscribe(SeelenEvent.SystemTrayChanged, (e) => {
  state.trayItems = e.payload;
});

// Order chosen by the user, as a list of `sortKey`s. Icons not in the list keep
// the order given by the system and go after the sorted ones.
export const iconsOrder = await persistentRune<string[]>("icons_order", []);

// Key that identifies an icon across restarts: the stable_id includes the
// window handle, which changes every time the owner app starts.
export function sortKey(icon: SysTrayIcon): string {
  if (icon.guid) {
    return icon.guid;
  }
  if (icon.exe_path) {
    return `${icon.exe_path.toLowerCase()}::${icon.uid}`;
  }
  return JSON.stringify(icon.stable_id);
}

export function sortIcons(icons: SysTrayIcon[], order: string[]): SysTrayIcon[] {
  const position = new Map(order.map((key, idx) => [key, idx]));
  return icons
    .map((icon, idx) => ({ icon, idx, pos: position.get(sortKey(icon)) }))
    .sort((a, b) => (a.pos ?? Infinity) - (b.pos ?? Infinity) || a.idx - b.idx)
    .map(({ icon }) => icon);
}
