import { $interactables, getWindowsForItem } from "./windows.ts";
import { $dock_state, HARDCODED_SEPARATOR_RIGHT } from "./items.ts";
import { effect } from "@preact/signals";
import { WegItemType } from "@seelen-ui/lib/types";
import type { AppOrFileWegItem, FolderWegItem } from "../types.ts";

effect(() => {
  const interactables = $interactables.value;
  const state = $dock_state.value;

  const appOrFileItems = state.items.filter(
    (item): item is AppOrFileWegItem => item.type === WegItemType.AppOrFile,
  );

  // Remove non-pinned items that have no windows
  const itemsToRemove = new Set(
    appOrFileItems
      .filter((item) => !item.pinned && getWindowsForItem(item, interactables).length === 0)
      .map((item) => item.id),
  );

  // Apps living inside folders also "cover" their windows, so we don't recreate
  // a top-level temporal item for an app that the user moved into a folder.
  const folderApps = state.items
    .filter((item): item is FolderWegItem => item.type === WegItemType.Folder)
    .flatMap((folder) => folder.items.map((entry) => ({ type: WegItemType.AppOrFile, ...entry }) as AppOrFileWegItem));

  // Find windows not covered by any remaining top-level item nor any folder item
  const remainingItems = appOrFileItems.filter((item) => !itemsToRemove.has(item.id));
  const coveringItems = [...remainingItems, ...folderApps];
  const uncoveredWindows = interactables.filter(
    (w) => !coveringItems.some((item) => getWindowsForItem(item, [w]).length > 0),
  );

  // Group by umid or process path to avoid duplicate items for the same app
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
  $dock_state.value = {
    ...state,
    items: [
      ...filteredItems.slice(0, separatorRightIdx),
      ...newItems,
      ...filteredItems.slice(separatorRightIdx),
    ],
  };
});

export * from "./settings.ts";
export * from "./windows.ts";
export * from "./system.ts";
export * from "./items.ts";
export * from "./hidden.ts";
