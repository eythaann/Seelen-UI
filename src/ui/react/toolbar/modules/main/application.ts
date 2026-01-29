import { type PluginId, type ToolbarItem, ToolbarJsScope } from "@seelen-ui/lib/types";

import { $toolbar_state } from "../shared/state/items.ts";

export function RestoreToDefault() {
  // based on src\background\state\application\toolbar_items.rs
  $toolbar_state.value = {
    ...$toolbar_state.value,
    left: [
      "@seelen/tb-user-menu" as PluginId,
      {
        id: crypto.randomUUID(),
        template: 'return "|"',
      } as ToolbarItem,
      "@default/focused-app" as PluginId,
      {
        id: crypto.randomUUID(),
        scopes: [ToolbarJsScope.FocusedApp],
        template: 'return focusedApp.title ? "-" : ""',
      } as ToolbarItem,
      "@default/focused-app-title" as PluginId,
    ],
    center: ["@seelen/tb-calendar-popup" as PluginId],
    right: [
      "@seelen/tb-system-tray" as PluginId,
      "@seelen/tb-keyboard-selector" as PluginId,
      "@seelen/keyboard-selector" as PluginId,
      "@seelen/tb-bluetooth-popup" as PluginId,
      "@seelen/tb-network-popup" as PluginId,
      "@seelen/tb-media-popup" as PluginId,
      "@default/power" as PluginId,
      "@seelen/tb-notifications" as PluginId,
      "@seelen/tb-quick-settings" as PluginId,
    ],
  };
}
