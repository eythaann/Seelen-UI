import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { StartMenuItem } from "@seelen-ui/lib/types";
import { lazyRune, persistentRune } from "libs/ui/svelte/utils";
import { StartDisplayMode, StartView } from "./constants";

const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
await subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

const startMenuItems = lazyRune(() => invoke(SeelenCommand.GetStartMenuItems));
await subscribe(SeelenEvent.StartMenuItemsChanged, startMenuItems.setByPayload);
await startMenuItems.init();

// Pinned items stored in localStorage
const PINNED_STORAGE_KEY = "seelen-apps-menu-pinned";

function loadPinnedItems(): string[] {
  try {
    const stored = localStorage.getItem(PINNED_STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function savePinnedItems(pinned: string[]) {
  localStorage.setItem(PINNED_STORAGE_KEY, JSON.stringify(pinned));
}

// Get unique identifier for an item
function getItemId(item: StartMenuItem): string {
  return item.umid || item.path;
}

// Pinned items state (array of item IDs)
let pinnedItemIds = $state<string[]>(loadPinnedItems());

const pinnedItems = $derived.by(() => {
  const items = startMenuItems.value;
  return pinnedItemIds
    .map((id) => items.find((item) => getItemId(item) === id))
    .filter((item): item is StartMenuItem => item !== undefined);
});

// Check if item is pinned
function isPinned(item: StartMenuItem): boolean {
  return pinnedItemIds.includes(getItemId(item));
}

// Toggle pin status
function togglePin(item: StartMenuItem) {
  const id = getItemId(item);
  if (isPinned(item)) {
    pinnedItemIds = pinnedItemIds.filter((pinnedId) => pinnedId !== id);
  } else {
    pinnedItemIds = [...pinnedItemIds, id];
  }
  savePinnedItems(pinnedItemIds);
}

// Verify pinned items still exist
function verifyPinnedItems() {
  const items = startMenuItems.value;
  const validIds = new Set(items.map(getItemId));
  const validPinned = pinnedItemIds.filter((id) => validIds.has(id));

  if (validPinned.length !== pinnedItemIds.length) {
    pinnedItemIds = validPinned;
    savePinnedItems(pinnedItemIds);
  }
}

$effect.root(() => {
  $effect(() => {
    startMenuItems.value;
    verifyPinnedItems();
  });
});

class State {
  isPinned = isPinned;
  togglePin = togglePin;
  getItemId = getItemId;

  #displayMode = persistentRune("StartDisplayMode", StartDisplayMode.Normal);
  #view = $state(StartView.Favorites);

  get monitors() {
    return monitors.value;
  }

  get pinnedItems() {
    return pinnedItems;
  }

  get allItems() {
    return startMenuItems.value;
  }

  get displayMode() {
    return this.#displayMode.value;
  }

  set displayMode(value: StartDisplayMode) {
    this.#displayMode.value = value;
  }

  get view() {
    return this.#view;
  }

  set view(value: StartView) {
    this.#view = value;
  }
}

export const globalState = new State();

/* $effect.root(() => {
  $effect(() => {
    if (globalState.displayMode === StartDisplayMode.Fullscreen) {
      Widget.getCurrent().webview.setShadow(false);
    } else {
      Widget.getCurrent().webview.setShadow(true);
    }
  });
});
 */
