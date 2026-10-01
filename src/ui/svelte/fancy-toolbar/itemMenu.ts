import { Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuCallbackPayload } from "@seelen-ui/lib/types";
import { toolbarActions } from "./state/items.svelte.ts";

const identifier = crypto.randomUUID();
const onItemMenuClick = "fancy-toolbar::item_menu_click";

Widget.self.webview.listen<ContextMenuCallbackPayload>(onItemMenuClick, ({ payload }) => {
  const { key, meta: itemId } = payload;
  if (typeof itemId !== "string") return;

  if (key === "remove") {
    toolbarActions.removeItem(itemId);
  }
});

export function getMenuForItem(t: (key: string) => string, itemId: string): ContextMenu {
  return {
    identifier,
    meta: itemId,
    items: [
      {
        type: "Item",
        key: "remove",
        label: t("context_menu.remove"),
        icon: "CgExtensionRemove",
        callbackEvent: onItemMenuClick,
      },
    ],
  };
}
