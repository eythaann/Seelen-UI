import { effect, signal } from "@preact/signals";
import { invoke, PluginList, SeelenCommand } from "@seelen-ui/lib";
import type { PluginId, ToolbarItem2, ToolbarState } from "@seelen-ui/lib/types";

import { matchIds } from "../utils.ts";
import { debounce } from "lodash";
import { path } from "@tauri-apps/api";
import yaml from "js-yaml";
import { fs } from "@seelen-ui/lib/tauri";
import { emit, listen } from "@tauri-apps/api/event";

export const $toolbar_state = signal(await invoke(SeelenCommand.StateGetToolbarItems));
listen("hidden::sync-toolbar-items", ({ payload }) => {
  // avoid infinite loop
  if (JSON.stringify(payload) !== JSON.stringify($toolbar_state.value)) {
    $toolbar_state.value = payload as ToolbarState;
  }
});

export const save = debounce(async (value: ToolbarState) => {
  console.trace("Saving toolbar state");
  const filePath = await path.join(await path.appDataDir(), "toolbar_items.yml");
  await fs.writeTextFile(filePath, yaml.dump(value));
}, 1000);

effect(() => {
  emit("hidden::sync-toolbar-items", $toolbar_state.value);
  save($toolbar_state.value);
});

export const $plugins = signal((await PluginList.getAsync()).forCurrentWidget());
await PluginList.onChange((list) => {
  $plugins.value = list.forCurrentWidget();
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
