import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SysTrayIcon } from "@seelen-ui/lib/types";
import { persistentRune } from "libs/ui/svelte/utils";

let trayItems = $state(await invoke(SeelenCommand.GetSystemTrayIcons));

subscribe(SeelenEvent.SystemTrayChanged, (e) => {
  trayItems = e.payload;
});

// Order chosen by the user, as a list of icon keys. Icons not in the list keep
// the order given by the system and go after the sorted ones.
const iconsOrder = await persistentRune<string[]>("icons_order", []);

class TrayState {
  get trayItems() {
    return trayItems;
  }
  get iconsOrder() {
    return iconsOrder.value;
  }
  set iconsOrder(value: string[]) {
    iconsOrder.value = value;
  }
}

export const trayState = new TrayState();

export interface KeyedIcon {
  key: string;
  icon: SysTrayIcon;
}

// Key that identifies an icon across restarts: the stable_id includes the
// window handle, which changes every time the owner app starts.
function baseKey(icon: SysTrayIcon): string {
  if (icon.guid) {
    return icon.guid;
  }
  if (icon.exe_path) {
    return `${icon.exe_path.toLowerCase()}::${icon.uid}`;
  }
  return JSON.stringify(icon.stable_id);
}

// A uid only has to be unique per window, so two instances of the same app can
// produce the same base key; repeated keys get a counter, in system order.
export function keyIcons(icons: SysTrayIcon[]): KeyedIcon[] {
  const seen = new Map<string, number>();
  return icons.map((icon) => {
    const base = baseKey(icon);
    const count = (seen.get(base) ?? 0) + 1;
    seen.set(base, count);
    return { key: count === 1 ? base : `${base}#${count}`, icon };
  });
}

export function sortIcons(icons: KeyedIcon[], order: string[]): KeyedIcon[] {
  const position = new Map(order.map((key, idx) => [key, idx]));
  return icons
    .map((item, idx) => ({ item, idx, pos: position.get(item.key) }))
    .sort((a, b) => (a.pos ?? Infinity) - (b.pos ?? Infinity) || a.idx - b.idx)
    .map(({ item }) => item);
}

// Writes the new order of the visible icons into the slots they already had in
// the saved order, so icons of apps that are not running keep their position.
// Visible icons that were never saved go at the end.
export function mergeOrder(saved: string[], visible: string[]): string[] {
  const visibleSet = new Set(visible);
  const pending = [...visible];
  const merged = saved.map((key) => (visibleSet.has(key) ? pending.shift()! : key));
  return [...merged, ...pending];
}
