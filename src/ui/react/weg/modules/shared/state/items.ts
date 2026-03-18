import { effect, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { type WegItem, type WegItems, WegItemType } from "@seelen-ui/lib/types";
import { debounce } from "lodash";

import type { SeparatorWegItem } from "../types.ts";
import { emit, listen } from "@tauri-apps/api/event";

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
  type: WegItemType.Separator,
};

export const HARDCODED_SEPARATOR_RIGHT: SeparatorWegItem = {
  id: "hardcoded-separator-2",
  type: WegItemType.Separator,
};

export const $dock_state = signal(getStateFromStored(await invoke(SeelenCommand.StateGetWegItems)));

subscribe(SeelenEvent.WegAddItem, (e) => {
  const item: WegItem = {
    ...e.payload,
    id: crypto.randomUUID(), // ensure uniqueness
    type: WegItemType.AppOrFile,
  };

  const items = [...$dock_state.value.items];
  const separatorIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
  items.splice(separatorIdx, 0, item);
  $dock_state.value = { ...$dock_state.value, items };
});

let isRemoteUpdate = false;
listen<SyncPayload>("hidden::sync-dock-items", ({ payload }) => {
  if (payload.source === CLIENT_ID) return;

  if (JSON.stringify(payload.state) !== JSON.stringify($dock_state.value)) {
    isRemoteUpdate = true;
    $dock_state.value = payload.state;
  }
});

const emitSyncEvent = debounce((items: OptimisticDockState) => {
  emit<SyncPayload>("hidden::sync-dock-items", {
    source: CLIENT_ID,
    state: items,
  });
}, 300);

const saveDockState = debounce(async (state: OptimisticDockState) => {
  console.trace("Saving dock state");

  const index1 = state.items.findIndex((i) => i.id === HARDCODED_SEPARATOR_LEFT.id);
  const index2 = state.items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);

  await invoke(SeelenCommand.StateWriteWegItems, {
    items: {
      isReorderDisabled: state.isReorderDisabled,
      left: state.items.slice(0, index1),
      center: state.items.slice(index1 + 1, index2),
      right: state.items.slice(index2 + 1),
    },
  });
}, 1000);

let mounted = false;
effect(() => {
  const state = $dock_state.value;

  // avoid writing on start of the widget
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

export const $dock_state_actions = {
  remove(idToRemove: string) {
    $dock_state.value = {
      ...$dock_state.value,
      items: $dock_state.value.items.filter((item) => item.id !== idToRemove),
    };
  },
  pinApp(id: string) {
    $dock_state.value = {
      ...$dock_state.value,
      items: $dock_state.value.items.map((item) => {
        if (item.id === id) {
          return { ...item, pinned: true };
        }
        return item;
      }),
    };
  },
  unpinApp(id: string) {
    $dock_state.value = {
      ...$dock_state.value,
      items: $dock_state.value.items.map((item) => {
        if (item.id === id) {
          return { ...item, pinned: false };
        }
        return item;
      }),
    };
  },
  addMediaModule() {
    if (!$dock_state.value.items.some((current) => current.type === WegItemType.Media)) {
      const newItems = [...$dock_state.value.items];
      newItems.push({
        id: crypto.randomUUID(),
        type: WegItemType.Media,
      });
      $dock_state.value = { ...$dock_state.value, items: newItems };
    }
  },
  addStartModule() {
    if (!$dock_state.value.items.some((current) => current.type === WegItemType.StartMenu)) {
      const newItems = [...$dock_state.value.items];
      newItems.unshift({
        id: crypto.randomUUID(),
        type: WegItemType.StartMenu,
      });
      $dock_state.value = { ...$dock_state.value, items: newItems };
    }
  },
  addDesktopModule() {
    if (!$dock_state.value.items.some((current) => current.type === WegItemType.ShowDesktop)) {
      const newItems = [...$dock_state.value.items];
      newItems.unshift({
        id: crypto.randomUUID(),
        type: WegItemType.ShowDesktop,
      });
      $dock_state.value = { ...$dock_state.value, items: newItems };
    }
  },
  addTrashBinModule() {
    if (!$dock_state.value.items.some((current) => current.type === WegItemType.TrashBin)) {
      const newItems = [...$dock_state.value.items];
      newItems.push({
        id: crypto.randomUUID(),
        type: WegItemType.TrashBin,
      });
      $dock_state.value = { ...$dock_state.value, items: newItems };
    }
  },
};
