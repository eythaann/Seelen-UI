import { SeelenCommand } from "@seelen-ui/lib";
import { Icon } from "@shared/components/Icon";
import { invoke } from "@tauri-apps/api/core";
import type { ItemType } from "antd/es/menu/interface";
import type { TFunction } from "i18next";

import type { SwItem } from "../../shared/types.ts";

import { $dock_state_actions } from "../../shared/state/items.ts";
import { WegItemType } from "@seelen-ui/lib/types";

export function getMenuForItem(t: TFunction, item: SwItem): ItemType[] {
  if (item.type === WegItemType.Media) {
    return [
      {
        key: "remove",
        label: t("media_menu.remove"),
        icon: <Icon iconName="CgExtensionRemove" />,
        onClick() {
          $dock_state_actions.remove(item.id);
        },
      },
    ];
  }

  if (item.type === WegItemType.StartMenu) {
    return [
      {
        key: "remove",
        label: t("start_menu.remove"),
        icon: <Icon iconName="CgExtensionRemove" />,
        onClick() {
          $dock_state_actions.remove(item.id);
        },
      },
    ];
  }

  // File or Folder pinned items
  if (item.type === WegItemType.Pinned) {
    return [
      {
        key: "remove",
        label: t("app_menu.unpin"),
        icon: <Icon iconName="RiUnpinLine" />,
        onClick() {
          $dock_state_actions.remove(item.id);
        },
      },
      {
        type: "divider",
      },
      {
        key: "weg_select_file_on_explorer",
        label: t("app_menu.open_file_location"),
        icon: <Icon iconName="MdOutlineMyLocation" />,
        onClick: () => invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path }),
      },
    ];
  }

  return [];
}
