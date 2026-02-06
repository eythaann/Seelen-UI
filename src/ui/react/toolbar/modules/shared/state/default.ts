import { type PluginId, type ToolbarItem, ToolbarJsScope } from "@seelen-ui/lib/types";
import { $toolbar_state } from "./items";

const baseItem: ToolbarItem = {
  id: "-",
  scopes: [],
  template: 'return ""',
  tooltip: null,
  badge: null,
  remoteData: {},
  style: {},
  onClick: null,
};

export function restoreStateToDefault() {
  // based on src\background\state\application\toolbar_items.rs
  $toolbar_state.value = {
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
        scopes: [ToolbarJsScope.FocusedApp],
        template: 'return focusedApp.title ? "-" : ""',
      },
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
