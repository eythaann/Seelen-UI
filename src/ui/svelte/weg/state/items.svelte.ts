import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { type WegItem, type WegItems, WegItemType } from "@seelen-ui/lib/types";
import { debounce } from "lodash";
import { emit, listen } from "@tauri-apps/api/event";
import type { SeparatorWegItem } from "../types.ts";

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

let _dockState = $state(getStateFromStored(await invoke(SeelenCommand.StateGetWegItems)));

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
    type: WegItemType.AppOrFile,
  };

  const items = [..._dockState.items];
  const separatorIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
  items.splice(separatorIdx, 0, item);
  _dockState = { ..._dockState, items };
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
    if (!_dockState.items.some((i) => i.type === WegItemType.Media)) {
      _dockState = {
        ..._dockState,
        items: [..._dockState.items, { id: crypto.randomUUID(), type: WegItemType.Media }],
      };
    }
  },
  addStartModule() {
    if (!_dockState.items.some((i) => i.type === WegItemType.StartMenu)) {
      _dockState = {
        ..._dockState,
        items: [{ id: crypto.randomUUID(), type: WegItemType.StartMenu }, ..._dockState.items],
      };
    }
  },
  addDesktopModule() {
    if (!_dockState.items.some((i) => i.type === WegItemType.ShowDesktop)) {
      _dockState = {
        ..._dockState,
        items: [{ id: crypto.randomUUID(), type: WegItemType.ShowDesktop }, ..._dockState.items],
      };
    }
  },
  addTrashBinModule() {
    if (!_dockState.items.some((i) => i.type === WegItemType.TrashBin)) {
      _dockState = {
        ..._dockState,
        items: [..._dockState.items, { id: crypto.randomUUID(), type: WegItemType.TrashBin }],
      };
    }
  },
};
