import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { PluginId, ToolbarItem, ToolbarItem2, ToolbarState } from "@seelen-ui/lib/types";
import { matchIds } from "../utils.ts";
import { debounce } from "lodash";
import { emit, listen } from "@tauri-apps/api/event";
import { plugins, toolbarItems as SaveFile } from "./getters.svelte.ts";

export { plugins };

export const baseItem: ToolbarItem = {
  id: "-",
  scopes: [],
  template: 'return ""',
  render: null,
  canvasSize: null,
  tooltip: null,
  badge: null,
  remoteData: {},
  style: {},
  onClick: null,
  onWheelUp: null,
  onWheelDown: null,
};

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

export function listToGroups(items: ToolbarItem2[], includeSeparator = false) {
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

function getStateFromStored(state: ToolbarState): OptimisticToolbarState {
  // check used for duplicate ids/plugins
  const seen = new Set<string>([HARDCODED_SEPARATOR_LEFT.id, HARDCODED_SEPARATOR_RIGHT.id]);
  const sanitize = (items: ToolbarItem2[]) =>
    items.filter((i) => {
      const id = typeof i === "string" ? i : i.id;
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    });

  return {
    isReorderDisabled: state.isReorderDisabled,
    items: [
      ...sanitize(state.left),
      HARDCODED_SEPARATOR_LEFT,
      ...sanitize(state.center),
      HARDCODED_SEPARATOR_RIGHT,
      ...sanitize(state.right),
    ],
  };
}

let _toolbarState = $state(getStateFromStored(SaveFile.value));

export const toolbarState = {
  get isReorderDisabled() {
    return _toolbarState.isReorderDisabled;
  },
  get items() {
    return _toolbarState.items;
  },
  set items(value: ToolbarItem2[]) {
    _toolbarState = { ..._toolbarState, items: value };
  },
  set state(value: OptimisticToolbarState) {
    _toolbarState = value;
  },
  get state() {
    return _toolbarState;
  },
};

export function restoreStateToDefault() {
  _toolbarState = getStateFromStored({
    isReorderDisabled: false,
    left: [
      "@seelen/tb-user-menu" as PluginId,
      {
        ...baseItem,
        id: crypto.randomUUID() as string,
        template: 'return "|"',
      },
      "@default/focused-app" as PluginId,
      {
        ...baseItem,
        id: crypto.randomUUID() as string,
        scopes: ["FocusedApp"],
        template: 'return focusedApp.title ? "-" : ""',
      },
      "@default/focused-app-title" as PluginId,
    ],
    center: ["@seelen/tb-calendar-popup" as PluginId],
    right: [
      "@seelen/tb-system-tray" as PluginId,
      "@seelen/tb-keyboard-selector" as PluginId,
      "@seelen/tb-bluetooth-popup" as PluginId,
      "@seelen/tb-network-popup" as PluginId,
      "@seelen/tb-media-popup" as PluginId,
      "@default/power" as PluginId,
      "@seelen/tb-notifications" as PluginId,
      "@seelen/tb-quick-settings" as PluginId,
    ],
  });
}

let isRemoteUpdate = false;
listen<SyncPayload>("hidden::sync-toolbar-items", ({ payload }) => {
  if (payload.source === CLIENT_ID) return;
  if (JSON.stringify(payload.state) !== JSON.stringify(_toolbarState)) {
    isRemoteUpdate = true;
    _toolbarState = payload.state;
  }
});

const emitSyncEvent = debounce((state: OptimisticToolbarState) => {
  emit<SyncPayload>("hidden::sync-toolbar-items", { source: CLIENT_ID, state });
}, 300);

const saveTbState = debounce(async (state: OptimisticToolbarState) => {
  console.trace("Saving toolbar state");
  const { left, center, right } = listToGroups(state.items);
  await invoke(SeelenCommand.StateWriteToolbarItems, {
    items: { isReorderDisabled: state.isReorderDisabled, left, center, right },
  });
}, 1000);

let mounted = false;
$effect.root(() => {
  $effect(() => {
    const state = _toolbarState;

    if (!mounted) {
      mounted = true;
      return;
    }

    if (isRemoteUpdate) {
      isRemoteUpdate = false;
      return;
    }

    const { left, center, right } = listToGroups(state.items);
    if (left.length === 0 && center.length === 0 && right.length === 0) {
      restoreStateToDefault();
      return;
    }

    emitSyncEvent(state);
    saveTbState(state);
  });
});

subscribe(SeelenEvent.PluginEnabled, (e) => {
  toolbarActions.addItem(e.payload);
});

$effect.root(() => {
  $effect(() => {
    const pluginIds = new Set(plugins.value.map((p) => p.id));
    const isStale = (item: ToolbarItem2) => typeof item === "string" && !pluginIds.has(item);

    if (!_toolbarState.items.some(isStale)) return;

    _toolbarState = {
      ..._toolbarState,
      items: _toolbarState.items.filter((item) => !isStale(item)),
    };
  });
});

export const toolbarActions = {
  addTextItem(text: string) {
    const cleaned = text.trim().replace(/"/g, '\\"');
    _toolbarState = {
      ..._toolbarState,
      items: [
        ..._toolbarState.items,
        {
          id: globalThis.crypto.randomUUID(),
          type: "text",
          template: `return "${cleaned}"`,
        } as any,
      ],
    };
  },
  addItem(id: PluginId) {
    if (!plugins.value.some((p) => p.id === id)) return;
    if (_toolbarState.items.some((item) => matchIds(item, id))) return;
    _toolbarState = {
      ..._toolbarState,
      items: [..._toolbarState.items, id],
    };
  },
  removeItem(id: string) {
    _toolbarState = {
      ..._toolbarState,
      items: _toolbarState.items.filter((item) => !matchIds(item, id)),
    };
  },
};
