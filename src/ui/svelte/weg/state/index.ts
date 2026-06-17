import { WegItemType } from "@seelen-ui/lib/types";
import { getWindowsForItem, interactables } from "./windows.svelte.ts";
import { dockState, HARDCODED_SEPARATOR_RIGHT } from "./items.svelte.ts";
import type { AppOrFileWegItem } from "../types.ts";

$effect.root(() => {
  $effect(() => {
    const windows = interactables.value;
    const state = dockState.state;

    const appOrFileItems = state.items.filter(
      (item): item is AppOrFileWegItem => item.type === WegItemType.AppOrFile,
    );

    const itemsToRemove = new Set(
      appOrFileItems
        .filter((item) => !item.pinned && getWindowsForItem(item, windows).length === 0)
        .map((item) => item.id),
    );

    const remainingItems = appOrFileItems.filter((item) => !itemsToRemove.has(item.id));
    const uncoveredWindows = windows.filter(
      (w) => !remainingItems.some((item) => getWindowsForItem(item, [w]).length > 0),
    );

    const seen = new Set<string>();
    const newItems: AppOrFileWegItem[] = [];

    for (const w of uncoveredWindows) {
      const key = w.umid ?? w.process.path?.toString();
      if (!key) continue;
      if (seen.has(key)) continue;
      seen.add(key);

      newItems.push({
        id: crypto.randomUUID(),
        type: WegItemType.AppOrFile,
        displayName: w.appName,
        umid: w.umid ?? null,
        path: w.process.path?.toString() ?? "",
        pinned: false,
        preventPinning: w.preventPinning,
        relaunch: w.relaunch ?? null,
      });
    }

    if (itemsToRemove.size === 0 && newItems.length === 0) return;

    const filteredItems = state.items.filter((item) => !itemsToRemove.has(item.id));
    const separatorRightIdx = filteredItems.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
    dockState.state = {
      ...state,
      items: [
        ...filteredItems.slice(0, separatorRightIdx),
        ...newItems,
        ...filteredItems.slice(separatorRightIdx),
      ],
    };
  });
});

export * from "./settings.svelte.ts";
export * from "./windows.svelte.ts";
export * from "./system.svelte.ts";
export * from "./items.svelte.ts";
export * from "./hidden.svelte.ts";
