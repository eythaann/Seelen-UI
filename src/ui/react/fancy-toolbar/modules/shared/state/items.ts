import { effect, signal } from "@preact/signals";
import { invoke, PluginList, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { PluginId, ToolbarItem, ToolbarItem2, ToolbarState } from "@seelen-ui/lib/types";

import { matchIds } from "../utils.ts";
import { debounce } from "lodash";
import { emit, listen } from "@tauri-apps/api/event";
import { baseItem, restoreStateToDefault } from "./default.ts";

export interface OptimisticToolbarState {
  isReorderDisabled: boolean;
  items: ToolbarItem2[];
}

interface SyncPayload {
  source: string;
  state: OptimisticToolbarState;
}

const CLIENT_ID = crypto.randomUUID();

export const HARDCODED_SEPARATOR_LEFT: ToolbarItem = {
  ...baseItem,
  id: "hardcoded-separator-1",
  template: 'return " "',
  style: { flexShrink: 0, opacity: 0 },
};

export const HARDCODED_SEPARATOR_RIGHT: ToolbarItem = {
  ...baseItem,
  id: "hardcoded-separator-2",
  template: 'return " "',
  style: { flexShrink: 0, opacity: 0 },
};

function splitItems(items: ToolbarItem2[]) {
  const idx1 = items.findIndex(
    (i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_LEFT.id,
  );
  const idx2 = items.findIndex(
    (i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_RIGHT.id,
  );
  return {
    left: items.slice(0, idx1),
    center: items.slice(idx1 + 1, idx2),
    right: items.slice(idx2 + 1),
  };
}

export function getStateFromStored(state: ToolbarState): OptimisticToolbarState {
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

export const $toolbar_state = signal(
  getStateFromStored(await invoke(SeelenCommand.StateGetToolbarItems)),
);

export const $plugins = signal((await PluginList.getAsync()).forCurrentWidget());
await PluginList.onChange((list) => {
  $plugins.value = list.forCurrentWidget();
});

let isRemoteUpdate = false;
listen<SyncPayload>("hidden::sync-toolbar-items", ({ payload }) => {
  if (payload.source === CLIENT_ID) return;

  if (JSON.stringify(payload.state) !== JSON.stringify($toolbar_state.value)) {
    isRemoteUpdate = true;
    $toolbar_state.value = payload.state;
  }
});

const emitSyncEvent = debounce((state: OptimisticToolbarState) => {
  emit<SyncPayload>("hidden::sync-toolbar-items", {
    source: CLIENT_ID,
    state,
  });
}, 300);

const saveTbState = debounce(async (state: OptimisticToolbarState) => {
  console.trace("Saving toolbar state");
  const { left, center, right } = splitItems(state.items);
  await invoke(SeelenCommand.StateWriteToolbarItems, {
    items: { isReorderDisabled: state.isReorderDisabled, left, center, right },
  });
}, 1000);

let mounted = false;
effect(() => {
  const state = $toolbar_state.value;

  // avoid writing on start of the widget
  if (!mounted) {
    mounted = true;
    return;
  }

  if (isRemoteUpdate) {
    isRemoteUpdate = false;
    return;
  }

  const { left, center, right } = splitItems(state.items);
  if (left.length === 0 && center.length === 0 && right.length === 0) {
    restoreStateToDefault();
    return;
  }

  emitSyncEvent(state);
  saveTbState(state);
});

subscribe(SeelenEvent.PluginEnabled, (e) => {
  if ($plugins.value.some((p) => p.id === e.payload)) {
    $actions.addItem(e.payload);
  }
});

export const $actions = {
  addTextItem(text: string) {
    const cleaned = text.trim().replace(/"/g, '\\"');
    $toolbar_state.value = {
      ...$toolbar_state.value,
      items: [
        ...$toolbar_state.value.items,
        {
          id: globalThis.crypto.randomUUID(),
          type: "text",
          template: `return "${cleaned}"`,
        } as any,
      ],
    };
  },
  addItem(id: PluginId) {
    $toolbar_state.value = {
      ...$toolbar_state.value,
      items: [...$toolbar_state.value.items, id],
    };
  },
  removeItem(id: string) {
    $toolbar_state.value = {
      ...$toolbar_state.value,
      items: $toolbar_state.value.items.filter((item) => !matchIds(item, id)),
    };
  },
};
