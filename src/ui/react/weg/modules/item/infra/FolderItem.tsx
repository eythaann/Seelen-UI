import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { FolderWegItem } from "../../shared/types.ts";

import { $dock_state_actions } from "../../shared/state/items.ts";
import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";

interface Props {
  item: FolderWegItem;
}

const PRESET_COLORS: Array<{ label: string; value: string | null; icon: string; iconColor: string }> = [
  { label: "Default", value: null, icon: "BsFolderFill", iconColor: "var(--system-accent-color)" },
  { label: "Blue", value: "#4a9eff", icon: "BsFolderFill", iconColor: "#4a9eff" },
  { label: "Green", value: "#4caf50", icon: "BsFolderFill", iconColor: "#4caf50" },
  { label: "Orange", value: "#ff9800", icon: "BsFolderFill", iconColor: "#ff9800" },
  { label: "Purple", value: "#9c27b0", icon: "BsFolderFill", iconColor: "#9c27b0" },
  { label: "Red", value: "#f44336", icon: "BsFolderFill", iconColor: "#f44336" },
  { label: "Pink", value: "#e91e8c", icon: "BsFolderFill", iconColor: "#e91e8c" },
  { label: "Yellow", value: "#ffeb3b", icon: "BsFolderFill", iconColor: "#ffeb3b" },
];

const identifier = crypto.randomUUID();
const colorSubmenuIdentifier = crypto.randomUUID();
const onFolderMenuClick = "weg::folder_menu_click";

let pendingFolderId: string | null = null;

Widget.self.webview.listen(onFolderMenuClick, ({ payload }) => {
  const { key } = payload as { key: string };
  if (!pendingFolderId) return;

  if (key === "delete_folder") {
    $dock_state_actions.deleteFolder(pendingFolderId);
  } else if (key.startsWith("color::")) {
    const color = key.slice(7) || null;
    $dock_state_actions.changeFolderColor(pendingFolderId, color);
  }

  pendingFolderId = null;
});

export const FolderItem = memo(({ item }: Props) => {
  const { t } = useTranslation();

  const iconColor = item.color ?? "var(--system-accent-color)";

  const onContextMenu = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      pendingFolderId = item.id;
      const { alignX, alignY } = getDockContextMenuAlignment($settings.value.position);
      invoke(SeelenCommand.TriggerContextMenu, {
        menu: {
          identifier,
          alignX,
          alignY,
          items: [
            {
              type: "Submenu",
              identifier: colorSubmenuIdentifier,
              icon: "IoColorPaletteOutline",
              label: t("folder_item.change_color", "Change Color"),
              items: PRESET_COLORS.map(({ label, value, icon, iconColor }) => ({
                type: "Item" as const,
                key: `color::${value ?? ""}`,
                icon,
                iconColor,
                label,
                callbackEvent: onFolderMenuClick,
              })),
            },
            { type: "Separator" },
            {
              type: "Item",
              key: "delete_folder",
              icon: "RiDeleteBin6Line",
              label: t("folder_item.delete_group", "Delete Group"),
              callbackEvent: onFolderMenuClick,
            },
          ],
        },
        forwardTo: null,
      });
    },
    [item, t],
  );

  return (
    <div
      className="weg-item weg-item-folder"
      onContextMenu={onContextMenu}
    >
      <Icon
        className="weg-item-icon weg-item-folder-icon"
        iconName="BsFolderFill"
        size="100%"
        style={{ color: iconColor }}
      />
    </div>
  );
});
