import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { dialog } from "@seelen-ui/lib/tauri";
import type {
  ContextMenu,
  ContextMenuCallbackPayload,
  ContextMenuItem,
  PluginId,
  WidgetId,
} from "@seelen-ui/lib/types";
import { getResourceText } from "libs/ui/react/utils/index.ts";
import { locale } from "./i18n/index.ts";
import { dockState, dockStateActions } from "./state/items.svelte.ts";
import { isHorizontalDock } from "./state/settings.svelte.ts";
import { plugins } from "./state/getters.svelte.ts";

const MENU_ID = crypto.randomUUID();
const PLUGINS_SUBMENU_ID = crypto.randomUUID();

const MENU_EVENT = "weg::bar_menu_click";
const PLUGINS_SUBMENU_EVENT = "weg::toggle_plugin";

let _t: (key: string) => string = (key) => key;

async function handleBarMenuClick({ key, checked, value }: ContextMenuCallbackPayload) {
  switch (key) {
    case "add-separator":
      dockStateActions.addSeparatorNear(value as { x: number; y: number });
      break;
    case "toggle-media-module":
      if (checked) {
        dockStateActions.addMediaModule();
      } else {
        dockStateActions.removeMediaModule();
      }

      break;
    case "reorder":
      dockState.state = {
        ...dockState.state,
        isReorderDisabled: !dockState.isReorderDisabled,
      };
      break;
    case "task_manager":
      invoke(SeelenCommand.OpenFile, { path: "Taskmgr.exe" });
      break;
    case "settings":
      invoke(SeelenCommand.TriggerWidget, {
        payload: { id: "@seelen/settings" as WidgetId },
      });
      break;
    case "add-item": {
      const files = await dialog.open({
        title: _t("taskbar_menu.add_file"),
        multiple: true,
        filters: [
          { name: "lnk", extensions: ["lnk"] },
          { name: "*", extensions: ["*"] },
        ],
      });
      for (const path of files || []) {
        await invoke(SeelenCommand.WegPinItem, { path });
      }
      break;
    }
  }
}

Widget.self.webview.listen<ContextMenuCallbackPayload>(MENU_EVENT, ({ payload }) => {
  handleBarMenuClick(payload);
});

Widget.self.webview.listen<ContextMenuCallbackPayload>(PLUGINS_SUBMENU_EVENT, ({ payload }) => {
  const { key, checked } = payload;
  if (checked) {
    dockStateActions.addPlugin(key as PluginId);
  } else {
    dockStateActions.removePlugin(key as PluginId);
  }
});

export function getSeelenWegMenu(
  t: (key: string) => string,
  cursor: { x: number; y: number },
): ContextMenu {
  _t = t;
  const { isReorderDisabled } = dockState;
  const language = locale.value;

  function isPluginAdded(id: PluginId): boolean {
    return dockState.items.some((item) => item.type === "Plugin" && item.plugin === id);
  }

  const pluginList = [
    {
      type: "Item",
      key: "toggle-media-module",
      icon: "PiMusicNotesPlusFill",
      label: t("taskbar_menu.media"),
      callbackEvent: MENU_EVENT,
      checked: dockState.items.some((i) => i.type === "Media"),
    },
    ...plugins.value.map((plugin) => ({
      type: "Item",
      key: plugin.id,
      label: getResourceText(plugin.metadata.displayName, language),
      icon: plugin.icon,
      callbackEvent: PLUGINS_SUBMENU_EVENT,
      checked: isPluginAdded(plugin.id),
    })),
  ].toSorted((p1, p2) => p1.label.localeCompare(p2.label)) as ContextMenuItem[];

  return {
    identifier: MENU_ID,
    items: [
      {
        type: "Submenu",
        icon: "CgExtensionAdd",
        label: t("taskbar_menu.modules"),
        identifier: PLUGINS_SUBMENU_ID,
        items: pluginList,
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "add-item",
        icon: "RiFileAddLine",
        label: t("taskbar_menu.add_file"),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "add-separator",
        icon: isHorizontalDock() ? "LuSeparatorVertical" : "LuSeparatorHorizontal",
        label: t("taskbar_menu.add_separator"),
        value: cursor,
        callbackEvent: MENU_EVENT,
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "reorder",
        icon: isReorderDisabled ? "CgLockUnlock" : "CgLock",
        label: t(
          isReorderDisabled ? "context_menu.reorder_enable" : "context_menu.reorder_disable",
        ),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "task_manager",
        icon: "PiChartLineFill",
        label: t("taskbar_menu.task_manager"),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "settings",
        icon: "RiSettings4Fill",
        label: t("taskbar_menu.settings"),
        callbackEvent: MENU_EVENT,
      },
    ],
  };
}
