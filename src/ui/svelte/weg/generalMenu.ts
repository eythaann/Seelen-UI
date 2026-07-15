import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuCallbackPayload, ContextMenuItem, WidgetId } from "@seelen-ui/lib/types";
import { WegItemType } from "@seelen-ui/lib/types";
import type { SwItem } from "./types.ts";
import { dockStateActions } from "./state/items.svelte.ts";
import { iconPackManager } from "libs/ui/svelte/components/Icon/index.ts";

const identifier = crypto.randomUUID();
const onItemMenuClick = "weg::item_menu_click";

let pendingItem: SwItem | null = null;

Widget.self.webview.listen<ContextMenuCallbackPayload>(onItemMenuClick, ({ payload }) => {
  const { key } = payload;
  const item = pendingItem;
  if (!item) return;

  if (key === "remove" || key === "unpin") {
    dockStateActions.remove(item.id);
  } else if (key === "open_location") {
    if ("path" in item) {
      invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path });
    }
  } else if (key === "empty_bin") {
    invoke(SeelenCommand.TrashBinEmpty);
  } else if (key === "edit_custom_icon") {
    const iconName = payload.value as string;
    const entry = iconPackManager.value.getCustomIconEntry(iconName);
    invoke(SeelenCommand.TriggerWidget, {
      payload: {
        id: "@seelen/icon-editor" as WidgetId,
        customArgs: { entry },
      },
    });
  }
});

export function getEditCustomIconEntry(
  t: (key: string) => string,
  iconName: string,
): ContextMenuItem {
  return {
    type: "Item",
    key: "edit_custom_icon",
    value: iconName,
    icon: "RiEditBoxLine",
    label: t("context_menu.edit_icon"),
    callbackEvent: onItemMenuClick,
  };
}

export function getMenuForItem(t: (key: string) => string, item: SwItem): ContextMenu {
  pendingItem = item;

  if (
    item.type === WegItemType.ShowDesktop ||
    item.type === WegItemType.Media ||
    item.type === WegItemType.StartMenu ||
    item.type === WegItemType.TrashBin ||
    item.type === WegItemType.Plugin
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
        getEditCustomIconEntry(t, "bin::full"),
        getEditCustomIconEntry(t, "bin::empty"),
        { type: "Separator" },
      );
    }

    if (item.type === WegItemType.StartMenu) {
      items.unshift(getEditCustomIconEntry(t, "@seelen/weg::start-menu"), { type: "Separator" });
    }

    if (item.type === WegItemType.ShowDesktop) {
      items.unshift(getEditCustomIconEntry(t, "@seelen/weg::show-desktop"), { type: "Separator" });
    }

    return { identifier, items };
  }

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
