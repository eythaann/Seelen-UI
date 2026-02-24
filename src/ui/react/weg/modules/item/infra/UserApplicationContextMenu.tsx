import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuItem } from "@seelen-ui/lib/types";
import type { TFunction } from "i18next";

import type { PinnedWegItem, TemporalWegItem } from "../../shared/types.ts";

import { $dock_state_actions } from "../../shared/state/items.ts";
import { $settings } from "../../shared/state/settings.ts";

const identifier = crypto.randomUUID();
const onAppMenuClick = "weg::app_menu_click";

let pendingAppItem: PinnedWegItem | TemporalWegItem | null = null;

Widget.self.webview.listen(onAppMenuClick, ({ payload }) => {
  const { key } = payload as { key: string };
  const item = pendingAppItem;
  if (!item) return;

  if (key === "unpin") {
    if (item.windows.length) {
      $dock_state_actions.unpinApp(item.id);
    } else {
      $dock_state_actions.remove(item.id);
    }
  } else if (key === "pin") {
    $dock_state_actions.pinApp(item.id);
  } else if (key === "run") {
    invoke(SeelenCommand.Run, {
      program: item.relaunchProgram,
      args: item.relaunchArgs,
      workingDir: item.relaunchIn,
      elevated: false,
    });
  } else if (key === "open_location") {
    invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path });
  } else if (key === "run_as") {
    invoke(SeelenCommand.Run, {
      program: item.relaunchProgram,
      args: item.relaunchArgs,
      workingDir: item.relaunchIn,
      elevated: true,
    });
  } else if (key === "copy_hwnd") {
    navigator.clipboard.writeText(
      JSON.stringify(item.windows.map((window) => window.handle.toString(16))),
    );
  } else if (key === "close") {
    item.windows.forEach((window) => {
      invoke(SeelenCommand.WegCloseApp, { hwnd: window.handle });
    });
  } else if (key === "kill") {
    item.windows.forEach((window) => {
      invoke(SeelenCommand.WegKillApp, { hwnd: window.handle });
    });
  }
});

export function getUserApplicationContextMenu(
  t: TFunction,
  item: PinnedWegItem | TemporalWegItem,
): ContextMenu {
  pendingAppItem = item;

  const isPinned = item.type === "Pinned";
  const items: ContextMenuItem[] = [];

  if (!item.pinDisabled) {
    if (isPinned) {
      items.push({
        type: "Item",
        key: "unpin",
        icon: "RiUnpinLine",
        label: t("app_menu.unpin"),
        callbackEvent: onAppMenuClick,
      });
    } else {
      items.push({
        type: "Item",
        key: "pin",
        icon: "RiPushpinLine",
        label: t("app_menu.pin"),
        callbackEvent: onAppMenuClick,
      });
    }
    items.push({ type: "Separator" });
  }

  items.push(
    {
      type: "Item",
      key: "run",
      icon: "IoOpenOutline",
      label: item.displayName,
      callbackEvent: onAppMenuClick,
    },
    {
      type: "Item",
      key: "open_location",
      icon: "MdOutlineMyLocation",
      label: t("app_menu.open_file_location"),
      callbackEvent: onAppMenuClick,
    },
    {
      type: "Item",
      key: "run_as",
      icon: "MdOutlineAdminPanelSettings",
      label: t("app_menu.run_as"),
      callbackEvent: onAppMenuClick,
    },
  );

  if (item.windows.length) {
    if ($settings.value.devTools) {
      items.push({
        type: "Item",
        key: "copy_hwnd",
        icon: "AiOutlineCopy",
        label: t("app_menu.copy_handles"),
        callbackEvent: onAppMenuClick,
      });
    }

    items.push({
      type: "Item",
      key: "close",
      icon: "BiWindowClose",
      label: item.windows.length > 1 ? t("app_menu.close_multiple") : t("app_menu.close"),
      callbackEvent: onAppMenuClick,
    });

    if ($settings.value.showEndTask) {
      items.push({
        type: "Item",
        key: "kill",
        icon: "MdOutlineDangerous",
        label: item.windows.length > 1 ? t("app_menu.kill_multiple") : t("app_menu.kill"),
        callbackEvent: onAppMenuClick,
      });
    }
  }

  return { identifier, items };
}
