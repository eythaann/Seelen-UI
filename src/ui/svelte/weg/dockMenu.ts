import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { dialog } from "@seelen-ui/lib/tauri";
import {
  type ContextMenu,
  type ContextMenuItem,
  type PluginId,
  WegItemType,
  type WidgetId,
} from "@seelen-ui/lib/types";
import { getResourceText } from "libs/ui/react/utils/index.ts";
import { locale } from "./i18n/index.ts";
import { dockState, dockStateActions } from "./state/items.svelte.ts";
import { plugins } from "./state/getters.svelte.ts";

const identifier = crypto.randomUUID();
const modulesIdentifier = crypto.randomUUID();
const onBarMenuClick = "weg::bar_menu_click";
const onTogglePlugin = "weg::toggle_plugin";

let _t: (key: string) => string = (key) => key;

type BarMenuKey =
  | "add-start-module"
  | "add-toggle-desktop-module"
  | "add-media-module"
  | "add-trash-bin-module"
  | "add-item"
  | "reorder"
  | "task_manager"
  | "settings";

async function handleBarMenuClick(key: BarMenuKey) {
  switch (key) {
    case "add-start-module":
      dockStateActions.addStartModule();
      break;
    case "add-toggle-desktop-module":
      dockStateActions.addDesktopModule();
      break;
    case "add-media-module":
      dockStateActions.addMediaModule();
      break;
    case "add-trash-bin-module":
      dockStateActions.addTrashBinModule();
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

Widget.self.webview.listen(onBarMenuClick, ({ payload }) => {
  handleBarMenuClick((payload as { key: BarMenuKey }).key);
});

Widget.self.webview.listen(onTogglePlugin, ({ payload }) => {
  const { key, checked } = payload as { key: PluginId; checked: boolean };
  if (checked) {
    dockStateActions.addPlugin(key);
  } else {
    dockStateActions.removePlugin(key);
  }
});

export function getSeelenWegMenu(t: (key: string) => string): ContextMenu {
  _t = t;
  const { isReorderDisabled } = dockState;
  const language = locale.value;

  function isPluginAdded(id: PluginId): boolean {
    return dockState.items.some((item) => item.type === WegItemType.Plugin && item.plugin === id);
  }

  return {
    identifier,
    items: [
      {
        type: "Submenu",
        icon: "CgExtensionAdd",
        label: t("taskbar_menu.modules"),
        identifier: modulesIdentifier,
        items: plugins.value
          .map<Extract<ContextMenuItem, { type: "Item" }>>((plugin) => ({
            type: "Item",
            key: plugin.id,
            label: getResourceText(plugin.metadata.displayName, language),
            icon: plugin.icon,
            callbackEvent: onTogglePlugin,
            checked: isPluginAdded(plugin.id),
          }))
          .toSorted((p1, p2) => p1.label.localeCompare(p2.label)),
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "add-start-module",
        icon: "BsWindows",
        label: t("taskbar_menu.start"),
        callbackEvent: onBarMenuClick,
      },
      {
        type: "Item",
        key: "add-toggle-desktop-module",
        icon: "IoDesktop",
        label: t("taskbar_menu.desktop"),
        callbackEvent: onBarMenuClick,
      },
      {
        type: "Item",
        key: "add-media-module",
        icon: "PiMusicNotesPlusFill",
        label: t("taskbar_menu.media"),
        callbackEvent: onBarMenuClick,
      },
      {
        type: "Item",
        key: "add-trash-bin-module",
        icon: "FaTrashAlt",
        label: t("taskbar_menu.trash_bin"),
        callbackEvent: onBarMenuClick,
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "add-item",
        icon: "RiFileAddLine",
        label: t("taskbar_menu.add_file"),
        callbackEvent: onBarMenuClick,
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "reorder",
        icon: isReorderDisabled ? "CgLockUnlock" : "CgLock",
        label: t(isReorderDisabled ? "context_menu.reorder_enable" : "context_menu.reorder_disable"),
        callbackEvent: onBarMenuClick,
      },
      {
        type: "Item",
        key: "task_manager",
        icon: "PiChartLineFill",
        label: t("taskbar_menu.task_manager"),
        callbackEvent: onBarMenuClick,
      },
      {
        type: "Item",
        key: "settings",
        icon: "RiSettings4Fill",
        label: t("taskbar_menu.settings"),
        callbackEvent: onBarMenuClick,
      },
    ],
  };
}
