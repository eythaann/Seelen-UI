import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { dialog } from "@seelen-ui/lib/tauri";
import type { ContextMenu } from "@seelen-ui/lib/types";
import type { TFunction } from "i18next";

import { WegItemType, type WidgetId } from "@seelen-ui/lib/types";

import { $dock_state, $dock_state_actions } from "../shared/state/items.ts";

const identifier = crypto.randomUUID();
const onBarMenuClick = "weg::bar_menu_click";

let _t: (key: string) => string = (key) => key;

Widget.self.webview.listen(onBarMenuClick, async ({ payload }) => {
  const { key } = payload as { key: string };

  if (key === "add-start-module") {
    $dock_state_actions.addStartModule();
  } else if (key === "add-toggle-desktop-module") {
    $dock_state_actions.addDesktopModule();
  } else if (key === "add-media-module") {
    $dock_state_actions.addMediaModule();
  } else if (key === "reorder") {
    $dock_state.value = {
      ...$dock_state.value,
      isReorderDisabled: !$dock_state.value.isReorderDisabled,
    };
  } else if (key === "task_manager") {
    invoke(SeelenCommand.OpenFile, { path: "Taskmgr.exe" });
  } else if (key === "settings") {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/settings" as WidgetId },
    });
  } else if (key === "add-item") {
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
  } else if (key === "add-folder") {
    const folder = await dialog.open({
      title: _t("taskbar_menu.add_folder"),
      directory: true,
    });
    if (folder) {
      await invoke(SeelenCommand.WegPinItem, { path: folder });
    }
  }
});

export function getSeelenWegMenu(t: TFunction): ContextMenu {
  _t = t;

  const isRestrictedBar = $dock_state.value.items.filter((c) => c.type !== WegItemType.Separator).length > 0 &&
    $dock_state.value.items.every((item) => item.type === WegItemType.Temporal && item.pinDisabled);

  if (isRestrictedBar) {
    return {
      identifier,
      items: [
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

  return {
    identifier,
    items: [
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
      { type: "Separator" },
      {
        type: "Item",
        key: "add-item",
        icon: "RiFileAddLine",
        label: t("taskbar_menu.add_file"),
        callbackEvent: onBarMenuClick,
      },
      {
        type: "Item",
        key: "add-folder",
        icon: "RiFolderAddLine",
        label: t("taskbar_menu.add_folder"),
        callbackEvent: onBarMenuClick,
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "reorder",
        icon: $dock_state.value.isReorderDisabled ? "CgLockUnlock" : "CgLock",
        label: t(
          $dock_state.value.isReorderDisabled ? "context_menu.reorder_enable" : "context_menu.reorder_disable",
        ),
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
