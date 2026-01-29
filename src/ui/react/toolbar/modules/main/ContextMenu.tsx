import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuItem, PluginId, WidgetId } from "@seelen-ui/lib/types";

import { useTranslation } from "react-i18next";

import { RestoreToDefault } from "./application.ts";

import { $actions, $plugins, $toolbar_state } from "../shared/state/items.ts";
import { getResourceText } from "@shared";

const identifier = crypto.randomUUID();
const modulesIdentifier = crypto.randomUUID();

const onContextMenuClick = "onContextMenuClick";
const onTogglePlugin = "onTogglePlugin";

Widget.self.webview.listen(onContextMenuClick, ({ payload }) => {
  const { key, checked: _checked } = payload as any;

  if (key === "reoder") {
    $toolbar_state.value = {
      ...$toolbar_state.value,
      isReorderDisabled: !$toolbar_state.value.isReorderDisabled,
    };
  }

  if (key === "task_manager") {
    invoke(SeelenCommand.OpenFile, { path: "Taskmgr.exe" });
  }

  if (key === "settings") {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/settings" as WidgetId },
    });
  }

  if (key === "restore") {
    RestoreToDefault();
  }
});

Widget.self.webview.listen(onTogglePlugin, ({ payload }) => {
  const { key: pluginId, checked } = payload as any;
  if (checked) {
    $actions.addItem(pluginId);
  } else {
    $actions.removeItem(pluginId);
  }
});

export function useMainContextMenu(): ContextMenu {
  const {
    t,
    i18n: { language },
  } = useTranslation();

  const allItems = [
    ...$toolbar_state.value.left,
    ...$toolbar_state.value.center,
    ...$toolbar_state.value.right,
  ];

  function isAlreadyAdded(id: PluginId): boolean {
    return allItems.some((item) => item === id);
  }

  return {
    identifier,
    items: [
      {
        type: "Submenu",
        icon: "CgExtensionAdd",
        label: t("context_menu.modules"),
        identifier: modulesIdentifier,
        items: [
          // restore
          {
            type: "Item",
            key: "restore",
            icon: "TbRestore",
            label: t("context_menu.restore"),
            callbackEvent: onContextMenuClick,
          },
          {
            type: "Separator",
          },
          ...$plugins.value
            .map<Extract<ContextMenuItem, { type: "Item" }>>((plugin) => ({
              type: "Item",
              key: plugin.id,
              label: getResourceText(plugin.metadata.displayName, language),
              icon: plugin.icon,
              callbackEvent: onTogglePlugin,
              checked: isAlreadyAdded(plugin.id),
            }))
            .toSorted((p1, p2) => p1.label.localeCompare(p2.label)),
        ],
      },
      {
        type: "Separator",
      },
      {
        type: "Item",
        key: "reoder",
        icon: $toolbar_state.value.isReorderDisabled ? "VscUnlock" : "VscLock",
        label: t(
          $toolbar_state.value.isReorderDisabled ? "context_menu.reorder_enable" : "context_menu.reorder_disable",
        ),
        callbackEvent: onContextMenuClick,
      },
      {
        type: "Item",
        key: "task_manager",
        icon: "PiChartLineFill",
        label: t("context_menu.task_manager"),
        callbackEvent: onContextMenuClick,
        checked: null,
        disabled: false,
      },
      {
        type: "Item",
        key: "settings",
        icon: "RiSettings4Fill",
        label: t("context_menu.settings"),
        callbackEvent: onContextMenuClick,
      },
    ],
  };
}
