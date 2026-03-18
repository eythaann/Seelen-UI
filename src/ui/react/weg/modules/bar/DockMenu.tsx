import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { dialog } from "@seelen-ui/lib/tauri";
import type { ContextMenu } from "@seelen-ui/lib/types";
import type { TFunction } from "i18next";

import type { WidgetId } from "@seelen-ui/lib/types";

import { $dock_state, $dock_state_actions } from "../shared/state/items.ts";

const identifier = crypto.randomUUID();
const onBarMenuClick = "weg::bar_menu_click";

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
      $dock_state_actions.addStartModule();
      break;

    case "add-toggle-desktop-module":
      $dock_state_actions.addDesktopModule();
      break;

    case "add-media-module":
      $dock_state_actions.addMediaModule();
      break;

    case "add-trash-bin-module":
      $dock_state_actions.addTrashBinModule();
      break;

    case "reorder":
      $dock_state.value = {
        ...$dock_state.value,
        isReorderDisabled: !$dock_state.value.isReorderDisabled,
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

export function getSeelenWegMenu(t: TFunction): ContextMenu {
  const { isReorderDisabled } = $dock_state.value;

  return {
    identifier,
    items: [
      // --- Add modules ---
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
      // --- File pinning ---
      {
        type: "Item",
        key: "add-item",
        icon: "RiFileAddLine",
        label: t("taskbar_menu.add_file"),
        callbackEvent: onBarMenuClick,
      },
      { type: "Separator" },
      // --- Dock controls ---
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
