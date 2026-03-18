import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuItem } from "@seelen-ui/lib/types";
import type { TFunction } from "i18next";

import { WegItemType } from "@seelen-ui/lib/types";

import type { SwItem } from "../../shared/types.ts";

import { $dock_state_actions } from "../../shared/state/items.ts";

const identifier = crypto.randomUUID();
const onItemMenuClick = "weg::item_menu_click";

let pendingItem: SwItem | null = null;

Widget.self.webview.listen(onItemMenuClick, ({ payload }) => {
  const { key } = payload as { key: string };
  const item = pendingItem;
  if (!item) return;

  if (key === "remove" || key === "unpin") {
    $dock_state_actions.remove(item.id);
  } else if (key === "open_location") {
    if ("path" in item) {
      invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path });
    }
  } else if (key === "empty_bin") {
    invoke(SeelenCommand.TrashBinEmpty);
  }
});

export function getMenuForItem(t: TFunction, item: SwItem): ContextMenu {
  pendingItem = item;

  if (
    item.type === WegItemType.ShowDesktop ||
    item.type === WegItemType.Media ||
    item.type === WegItemType.StartMenu ||
    item.type === WegItemType.TrashBin
  ) {
    const items: ContextMenuItem[] = [
      {
        type: "Item",
        key: "remove",
        icon: "CgExtensionRemove",
        label: t("context_menu.remove_module"),
        callbackEvent: onItemMenuClick,
      },
    ];

    if (item.type === WegItemType.TrashBin) {
      items.unshift(
        {
          type: "Item",
          key: "empty_bin",
          icon: "FaRegTrashAlt",
          label: t("trash_bin.empty_bin"),
          callbackEvent: onItemMenuClick,
        },
        { type: "Separator" },
      );
    }

    return {
      identifier,
      items,
    };
  }

  // File or Folder pinned items
  if (item.type === WegItemType.AppOrFile) {
    return {
      identifier,
      items: [
        {
          type: "Item",
          key: "unpin",
          icon: "RiUnpinLine",
          label: t("app_menu.unpin"),
          callbackEvent: onItemMenuClick,
        },
        { type: "Separator" },
        {
          type: "Item",
          key: "open_location",
          icon: "MdOutlineMyLocation",
          label: t("app_menu.open_file_location"),
          callbackEvent: onItemMenuClick,
        },
      ],
    };
  }

  return { identifier, items: [] };
}
