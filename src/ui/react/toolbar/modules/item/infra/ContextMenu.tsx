import type { ToolbarItem } from "@seelen-ui/lib/types";
import { Icon } from "@shared/components/Icon";
import type { TFunction } from "i18next";

import { $actions } from "../../shared/state/items.ts";

export function CommonItemContextMenu(
  t: TFunction,
  item: Omit<ToolbarItem, "type">,
) {
  return [
    {
      key: "remove",
      label: t("context_menu.remove"),
      icon: <Icon iconName="CgExtensionRemove" />,
      className: "ft-bar-item-context-menu-item",
      onClick() {
        $actions.removeItem(item.id);
      },
    },
  ];
}
