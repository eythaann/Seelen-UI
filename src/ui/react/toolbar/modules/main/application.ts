import { fs } from "@seelen-ui/lib/tauri";
import { type PluginId, type ToolbarItem, ToolbarModuleType } from "@seelen-ui/lib/types";
import { path } from "@tauri-apps/api";
import yaml from "js-yaml";
import { debounce } from "lodash";

import { $toolbar_state } from "../shared/state/items.ts";

$toolbar_state.subscribe(debounce(async (value) => {
  const filePath = await path.join(
    await path.appDataDir(),
    "toolbar_items.yml",
  );
  await fs.writeTextFile(filePath, yaml.dump(value));
}, 1000));

export function RestoreToDefault() {
  // based on src\background\state\application\toolbar_items.rs
  $toolbar_state.value = {
    ...$toolbar_state.value,
    left: [
      "@default/user-folder" as PluginId,
      {
        id: crypto.randomUUID(),
        type: ToolbarModuleType.Text,
        template: 'return "|"',
      } as ToolbarItem,
      "@default/focused-app" as PluginId,
      {
        id: crypto.randomUUID(),
        type: ToolbarModuleType.Generic,
        template: 'return window.title ? "-" : ""',
      } as ToolbarItem,
      "@default/focused-app-title" as PluginId,
    ],
    center: ["@default/date" as PluginId],
    right: [
      "@default/system-tray" as PluginId,
      "@default/keyboard" as PluginId,
      "@default/bluetooth" as PluginId,
      "@default/network" as PluginId,
      "@default/media" as PluginId,
      "@default/power" as PluginId,
      "@default/notifications" as PluginId,
      "@default/quick-settings" as PluginId,
    ],
  };
}
