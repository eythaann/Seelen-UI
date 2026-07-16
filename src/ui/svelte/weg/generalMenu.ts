import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuCallbackPayload, ContextMenuItem, WidgetId } from "@seelen-ui/lib/types";
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

  if (key === "remove") {
    dockStateActions.remove(item.id);
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

export function getEmptyTrashBinEntry(t: (key: string) => string): ContextMenuItem {
  return {
    type: "Item",
    key: "empty_bin",
    icon: "FaRegTrashAlt",
    label: t("trash_bin.empty_bin"),
    callbackEvent: onItemMenuClick,
  };
}

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

  if (item.type === "Media" || item.type === "Plugin") {
    const items: ContextMenuItem[] = [
      {
        type: "Item",
        key: "remove",
        icon: "CgExtensionRemove",
        label: t("context_menu.remove_module"),
        callbackEvent: onItemMenuClick,
      },
    ];

    return { identifier, items };
  }

  return { identifier, items: [] };
}
