import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { PluginId, WegItem, WegItems } from "@seelen-ui/lib/types";
import { debounce } from "lodash";
import { emit, listen } from "@tauri-apps/api/event";
import type { AppOrFileWegItem, SeparatorWegItem } from "../types.ts";
import { getWindowsForItem, interactables } from "./windows.svelte.ts";
import { plugins, wegItems } from "./getters.svelte.ts";
import { isHorizontalDock } from "./settings.svelte.ts";

interface OptimisticDockState {
  isReorderDisabled: boolean;
  items: WegItem[];
}

interface SyncPayload {
  source: string;
  state: OptimisticDockState;
}

const CLIENT_ID = crypto.randomUUID();

export const HARDCODED_SEPARATOR_LEFT: SeparatorWegItem = {
  id: "hardcoded-separator-1",
  type: "Separator",
};

export const HARDCODED_SEPARATOR_RIGHT: SeparatorWegItem = {
  id: "hardcoded-separator-2",
  type: "Separator",
};

function getStateFromStored(state: WegItems): OptimisticDockState {
  return {
    isReorderDisabled: state.isReorderDisabled,
    items: [
      ...state.left,
      HARDCODED_SEPARATOR_LEFT,
      ...state.center,
      HARDCODED_SEPARATOR_RIGHT,
      ...state.right,
    ],
  };
}

let _dockState = $state(getStateFromStored(wegItems.value));

export const dockState = {
  get isReorderDisabled() {
    return _dockState.isReorderDisabled;
  },
  get items() {
    return _dockState.items;
  },
  get state() {
    return _dockState;
  },
  set state(value: OptimisticDockState) {
    _dockState = value;
  },
  set items(value: WegItem[]) {
    _dockState = { ..._dockState, items: value };
  },
};

subscribe(SeelenEvent.WegAddItem, (e) => {
  const item: WegItem = {
    ...e.payload,
    id: crypto.randomUUID(),
    type: "AppOrFile",
  };

  const items = [..._dockState.items];
  const separatorIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
  items.splice(separatorIdx, 0, item);
  _dockState = { ..._dockState, items };
});

subscribe(SeelenEvent.PluginEnabled, (e) => {
  dockStateActions.addPlugin(e.payload);
});

let isRemoteUpdate = false;
listen<SyncPayload>("hidden::sync-dock-items", ({ payload }) => {
  if (payload.source === CLIENT_ID) return;
  if (JSON.stringify(payload.state) !== JSON.stringify(_dockState)) {
    isRemoteUpdate = true;
    _dockState = payload.state;
  }
});

const emitSyncEvent = debounce((state: OptimisticDockState) => {
  emit<SyncPayload>("hidden::sync-dock-items", { source: CLIENT_ID, state });
}, 300);

export function listToGroups(items: WegItem[], includeSeparator = false) {
  let idx1 = items.findIndex((i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_LEFT.id);
  let idx2 = items.findIndex((i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_RIGHT.id);
  if (idx1 > idx2) {
    [idx1, idx2] = [idx2, idx1];
  }
  return {
    left: items.slice(0, idx1),
    center: includeSeparator ? items.slice(idx1, idx2 + 1) : items.slice(idx1 + 1, idx2),
    right: items.slice(idx2 + 1),
  };
}

const saveDockState = debounce(async (state: OptimisticDockState) => {
  console.trace("Saving dock state");
  await invoke(SeelenCommand.StateWriteWegItems, {
    items: {
      isReorderDisabled: state.isReorderDisabled,
      ...listToGroups(state.items),
    },
  });
}, 1000);

let mounted = false;
$effect.root(() => {
  $effect(() => {
    const state = _dockState;

    if (!mounted) {
      mounted = true;
      return;
    }

    if (isRemoteUpdate) {
      isRemoteUpdate = false;
      return;
    }

    emitSyncEvent(state);
    saveDockState(state);
  });
});

export const dockStateActions = {
  remove(idToRemove: string) {
    _dockState = {
      ..._dockState,
      items: _dockState.items.filter((item) => item.id !== idToRemove),
    };
  },
  pinApp(id: string) {
    _dockState = {
      ..._dockState,
      items: _dockState.items.map((item) => (item.id === id ? { ...item, pinned: true } : item)),
    };
  },
  unpinApp(id: string) {
    _dockState = {
      ..._dockState,
      items: _dockState.items.map((item) => (item.id === id ? { ...item, pinned: false } : item)),
    };
  },
  addMediaModule() {
    if (!_dockState.items.some((i) => i.type === "Media")) {
      _dockState = {
        ..._dockState,
        items: [..._dockState.items, { id: crypto.randomUUID(), type: "Media" }],
      };
    }
  },
  removeMediaModule() {
    _dockState = {
      ..._dockState,
      items: _dockState.items.filter((i) => i.type !== "Media"),
    };
  },
  addPlugin(plugin: PluginId) {
    if (!plugins.value.some((p) => p.id === plugin)) return;
    if (_dockState.items.some((i) => i.type === "Plugin" && i.plugin === plugin)) return;
    _dockState = {
      ..._dockState,
      items: [..._dockState.items, { id: crypto.randomUUID(), type: "Plugin", plugin }],
    };
  },
  removePlugin(plugin: PluginId) {
    _dockState = {
      ..._dockState,
      items: _dockState.items.filter((i) => !(i.type === "Plugin" && i.plugin === plugin)),
    };
  },
  /** Inserts a new separator next to whichever rendered item is closest to the given cursor position. */
  addSeparatorNear(cursor: { x: number; y: number }) {
    const newSeparator: SeparatorWegItem = { id: crypto.randomUUID(), type: "Separator" };
    const items = [..._dockState.items];

    const containers = Array.from(
      document.querySelectorAll<HTMLElement>(".weg-item-drag-container"),
    );

    let insertIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);

    if (containers.length > 0) {
      const horizontal = isHorizontalDock();
      let nearestId: string | undefined;
      let nearestDist = Infinity;
      let insertAfter = false;

      for (const el of containers) {
        const rect = el.getBoundingClientRect();
        const cx = rect.left + rect.width / 2;
        const cy = rect.top + rect.height / 2;
        const dist = Math.hypot(cursor.x - cx, cursor.y - cy);
        if (dist < nearestDist) {
          nearestDist = dist;
          nearestId = el.dataset.itemId;
          insertAfter = horizontal ? cursor.x > cx : cursor.y > cy;
        }
      }

      const nearestIdx = items.findIndex((i) => i.id === nearestId);
      if (nearestIdx !== -1) {
        insertIdx = insertAfter ? nearestIdx + 1 : nearestIdx;
      }
    }

    items.splice(insertIdx, 0, newSeparator);
    _dockState = { ..._dockState, items };
  },
};

$effect.root(() => {
  $effect(() => {
    const pluginIds = new Set(plugins.value.map((p) => p.id));
    const hasStalePlugin = _dockState.items.some(
      (item) => item.type === "Plugin" && !pluginIds.has(item.plugin),
    );
    if (!hasStalePlugin) return;

    _dockState = {
      ..._dockState,
      items: _dockState.items.filter(
        (item) => !(item.type === "Plugin" && !pluginIds.has(item.plugin)),
      ),
    };
  });
});

$effect.root(() => {
  $effect(() => {
    const windows = interactables.value;
    const state = _dockState;

    const appOrFileItems = state.items.filter(
      (item): item is AppOrFileWegItem => item.type === "AppOrFile",
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
        type: "AppOrFile",
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
    _dockState = {
      ...state,
      items: [
        ...filteredItems.slice(0, separatorRightIdx),
        ...newItems,
        ...filteredItems.slice(separatorRightIdx),
      ],
    };
  });
});
