import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { StartMenuItem, StartMenuLayoutItem } from "@seelen-ui/lib/types";
import { lazyRune, persistentRune } from "libs/ui/svelte/utils";
import { StartDisplayMode, StartView } from "../constants";
import { foldersAsStartMenuItems } from "./knownFolders.svelte";

import { locale } from "../i18n/index.ts";

let settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));

const user = lazyRune(() => invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, user.setByPayload);

const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

const startMenuItems = lazyRune(() => invoke(SeelenCommand.GetStartMenuItems));
subscribe(SeelenEvent.StartMenuItemsChanged, startMenuItems.setByPayload);

await Promise.all([settings.init(), user.init(), monitors.init(), startMenuItems.init()]);

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language || "en");
  });
});

// Folder and pinned items types
export interface FavFolderItem {
  type: "folder";
  itemId: string;
  name: string;
  itemIds: string[];
}

export interface FavAppItem {
  type: "app";
  itemId: string;
}

export type FavPinnedItem = FavAppItem | FavFolderItem;

const initialState: FavPinnedItem[] = [];

try {
  const layout = await invoke(SeelenCommand.GetNativeStartMenu);
  for (const _item of layout.pinnedList) {
    const item = _item as StartMenuLayoutItem as any;
    if (item.desktopAppLink) {
      let path = item.desktopAppLink.toLowerCase();
      const found = startMenuItems.value.find((i) => i.path.toLowerCase() === path);

      if (found) {
        initialState.push({
          type: "app",
          itemId: getItemId(found),
        });
        continue;
      }
    }

    initialState.push({
      type: "app",
      itemId: item.packagedAppId || item.destopAppId || item.desktopAppLink,
    });
  }
} catch (error) {
  console.error("Failed to get native pinned items:", error);
}

const pinnedItems = await persistentRune<FavPinnedItem[]>("favorites", initialState);

// Get unique identifier for an item
function getItemId(item: StartMenuItem): string {
  return item.umid || item.path.toLowerCase();
}

// Check if an item is pinned
function isPinned(item: StartMenuItem): boolean {
  const itemId = getItemId(item);
  return pinnedItems.value.some((pinned) => {
    if (pinned.type === "app") {
      return pinned.itemId === itemId;
    } else {
      return pinned.itemIds.includes(itemId);
    }
  });
}

// Toggle pin status of an item
function togglePin(item: StartMenuItem): void {
  const itemId = getItemId(item);
  const currentPinned = pinnedItems.value;

  if (isPinned(item)) {
    // Unpin: remove from apps or folders
    pinnedItems.value = currentPinned
      .map((pinned) => {
        if (pinned.type === "app") {
          return pinned.itemId === itemId ? null : pinned;
        } else {
          const newItemIds = pinned.itemIds.filter((id) => id !== itemId);
          // Remove folder if it has less than 2 items
          if (newItemIds.length < 2) {
            return null;
          }
          return { ...pinned, itemIds: newItemIds };
        }
      })
      .filter((item): item is FavPinnedItem => item !== null);
  } else {
    // Pin: add as app item
    pinnedItems.value = [...currentPinned, { type: "app", itemId: itemId }];
  }
}

// Update entire pinned items array
function updatePinnedItems(items: FavPinnedItem[]): void {
  pinnedItems.value = items;
}

// Create a new folder from two app items at a specific position
function createFolder(itemId1: string, itemId2: string, targetIdx?: number): FavFolderItem {
  const newFolder: FavFolderItem = {
    type: "folder",
    itemId: crypto.randomUUID(),
    name: "",
    itemIds: [itemId1, itemId2],
  };

  // Remove both items if they exist as standalone apps
  const filtered = pinnedItems.value.filter(
    (item) => !(item.type === "app" && (item.itemId === itemId1 || item.itemId === itemId2)),
  );

  // Insert folder at target position or at end
  if (targetIdx !== undefined && targetIdx >= 0 && targetIdx <= filtered.length) {
    const before = filtered.slice(0, targetIdx);
    const after = filtered.slice(targetIdx);
    pinnedItems.value = [...before, newFolder, ...after];
  } else {
    pinnedItems.value = [...filtered, newFolder];
  }

  return newFolder;
}

// Add an item to an existing folder
function addItemToFolder(folderId: string, itemId: string): void {
  pinnedItems.value = pinnedItems.value.map((pinned) => {
    if (pinned.type === "folder" && pinned.itemId === folderId) {
      // Check if item already exists in folder
      if (!pinned.itemIds.includes(itemId)) {
        return { ...pinned, itemIds: [...pinned.itemIds, itemId] };
      }
    }
    return pinned;
  });

  // Remove the item if it exists as standalone app
  pinnedItems.value = pinnedItems.value.filter(
    (item) => !(item.type === "app" && item.itemId === itemId),
  );
}

// Update folder properties
function updateFolder(
  folderId: string,
  updates: Partial<Omit<FavFolderItem, "type" | "id">>,
): void {
  pinnedItems.value = pinnedItems.value.map((pinned) => {
    if (pinned.type === "folder" && pinned.itemId === folderId) {
      return { ...pinned, ...updates };
    }
    return pinned;
  });
}

// Merge source folder into target folder
function mergeFolders(sourceFolderId: string, targetFolderId: string): void {
  const sourceFolder = pinnedItems.value.find(
    (item) => item.type === "folder" && item.itemId === sourceFolderId,
  ) as FavFolderItem | undefined;

  if (!sourceFolder) {
    return;
  }

  // Add all items from source folder to target folder
  pinnedItems.value = pinnedItems.value.map((pinned) => {
    if (pinned.type === "folder" && pinned.itemId === targetFolderId) {
      // Merge items, avoiding duplicates
      const mergedItemIds = [...new Set([...pinned.itemIds, ...sourceFolder.itemIds])];
      return { ...pinned, itemIds: mergedItemIds };
    }
    return pinned;
  });

  // Remove the source folder
  pinnedItems.value = pinnedItems.value.filter(
    (item) => !(item.type === "folder" && item.itemId === sourceFolderId),
  );
}

// Disband a folder, converting all items to individual apps
function disbandFolder(folderId: string): void {
  const folder = pinnedItems.value.find(
    (item) => item.type === "folder" && item.itemId === folderId,
  ) as FavFolderItem | undefined;

  if (!folder) {
    return;
  }

  const folderIndex = pinnedItems.value.findIndex((item) => item.itemId === folderId);
  const withoutFolder = pinnedItems.value.filter((item) => item.itemId !== folderId);
  const newApps: FavAppItem[] = folder.itemIds.map((itemId) => ({
    type: "app",
    itemId,
  }));

  pinnedItems.value = [
    ...withoutFolder.slice(0, folderIndex),
    ...newApps,
    ...withoutFolder.slice(folderIndex),
  ];
}

// Verify pinned items still exist
function verifyPinnedItems() {
  const items = startMenuItems.value;
  const folderItems = foldersAsStartMenuItems.value;
  const validIds = new Set([...items, ...folderItems].map(getItemId));

  // Path-based items (manually pinned arbitrary files) are always kept.
  function isValid(id: string): boolean {
    return validIds.has(id) || id.includes("\\") || id.includes("/");
  }

  const current = pinnedItems.value;
  const validPinned: FavPinnedItem[] = [];

  for (const pinned of current) {
    if (pinned.type === "app") {
      if (isValid(pinned.itemId)) validPinned.push(pinned);
    } else {
      const validItemIds = pinned.itemIds.filter(isValid);
      if (validItemIds.length >= 2) validPinned.push({ ...pinned, itemIds: validItemIds });
    }
  }

  if (validPinned.length !== current.length) {
    pinnedItems.value = validPinned;
  }
}

$effect.root(() => {
  $effect(() => {
    verifyPinnedItems();
  });
});

const displayMode = await persistentRune("display_mode", StartDisplayMode.Normal);
class State {
  isPinned = isPinned;
  togglePin = togglePin;
  getItemId = getItemId;
  updatePinnedItems = updatePinnedItems;
  createFolder = createFolder;
  addItemToFolder = addItemToFolder;
  updateFolder = updateFolder;
  mergeFolders = mergeFolders;
  disbandFolder = disbandFolder;

  desiredMonitorId = $state<string | null>(null);

  view = $state(StartView.Favorites);
  preselectedItem = $state<string | null>(null);

  version = $state<number>(0);

  get user() {
    return user.value;
  }

  get monitors() {
    return monitors.value;
  }

  get pinnedItems() {
    return pinnedItems.value;
  }

  set pinnedItems(value: FavPinnedItem[]) {
    pinnedItems.value = value;
  }

  get allItems() {
    return startMenuItems.value;
  }

  get displayMode() {
    return displayMode.value;
  }

  set displayMode(value: StartDisplayMode) {
    displayMode.value = value;
  }

  // Get StartMenuItem by ID
  getMenuItem(id: string): StartMenuItem | undefined {
    const fromStart = this.allItems.find((item) => getItemId(item) === id);
    if (fromStart) return fromStart;

    const fromFolders = foldersAsStartMenuItems.value.find((item) => getItemId(item) === id);
    if (fromFolders) return fromFolders;

    // Synthesize an item for arbitrary path-based pins (e.g. pinned from dock)
    if (id.includes("\\") || id.includes("/")) {
      return {
        path: id,
        umid: null,
        display_name: id.split(/[\\/]/g).pop() || id,
        target: null,
        toast_activator: null,
      };
    }

    return undefined;
  }
}

export const globalState = new State();
