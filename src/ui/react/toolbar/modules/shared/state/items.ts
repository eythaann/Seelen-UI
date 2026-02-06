import { effect, signal } from "@preact/signals";
import { invoke, PluginList, SeelenCommand } from "@seelen-ui/lib";
import type { PluginId, ToolbarItem2, ToolbarState } from "@seelen-ui/lib/types";

import { matchIds } from "../utils.ts";
import { debounce } from "lodash";
import { emit, listen } from "@tauri-apps/api/event";
import { restoreStateToDefault } from "./default.ts";

export const $toolbar_state = signal(await invoke(SeelenCommand.StateGetToolbarItems));
listen("hidden::sync-toolbar-items", ({ payload }) => {
  // avoid infinite loop
  if (JSON.stringify(payload) !== JSON.stringify($toolbar_state.value)) {
    $toolbar_state.value = payload as ToolbarState;
  }
});

export const $plugins = signal((await PluginList.getAsync()).forCurrentWidget());
await PluginList.onChange((list) => {
  $plugins.value = list.forCurrentWidget();
});

export const saveTbState = debounce(async (items: ToolbarState) => {
  console.trace("Saving toolbar state");
  await invoke(SeelenCommand.StateWriteToolbarItems, { items });
}, 1000);

effect(() => {
  if (
    $toolbar_state.value.left.length === 0 &&
    $toolbar_state.value.center.length === 0 &&
    $toolbar_state.value.right.length === 0
  ) {
    restoreStateToDefault();
    return;
  }

  emit("hidden::sync-toolbar-items", $toolbar_state.value);
  saveTbState($toolbar_state.value);
});

export const $actions = {
  addTextItem(text: string) {
    const cleaned = text.trim().replace(/"/g, '\\"');
    const newRight = [...$toolbar_state.value.right];
    newRight.push({
      id: globalThis.crypto.randomUUID(),
      type: "text",
      template: `return "${cleaned}"`,
    } as any);
    $toolbar_state.value = { ...$toolbar_state.value, right: newRight };
  },
  addItem(id: PluginId) {
    const { left, center, right } = $toolbar_state.value;
    const alreadyExists = left.includes(id) || right.includes(id) || center.includes(id);
    if (!alreadyExists) {
      $toolbar_state.value = {
        ...$toolbar_state.value,
        right: [...right, id],
      };
    }
  },
  removeItem(id: string) {
    let filter = (d: ToolbarItem2) => !matchIds(d, id);
    const { left, center, right, ...rest } = $toolbar_state.value;
    $toolbar_state.value = {
      ...rest,
      left: left.filter(filter),
      center: center.filter(filter),
      right: right.filter(filter),
    };
  },
};
