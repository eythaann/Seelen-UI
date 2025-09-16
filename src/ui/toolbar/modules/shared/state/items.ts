import { signal } from "@preact/signals";
import { invoke, PluginList, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { PluginId, ToolbarItem2 } from "@seelen-ui/lib/types";

import { matchIds } from "../utils";

export const $toolbar_state = signal(
  await invoke(SeelenCommand.StateGetToolbarItems),
);
subscribe(
  SeelenEvent.StateToolbarItemsChanged,
  (event) => ($toolbar_state.value = event.payload),
);

export const $plugins = signal(
  (await PluginList.getAsync()).forCurrentWidget(),
);
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
    const alreadyExists = left.includes(id) || right.includes(id) ||
      center.includes(id);
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
