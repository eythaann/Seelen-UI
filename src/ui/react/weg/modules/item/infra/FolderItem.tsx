import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { SeelenWegSide, WegItemType } from "@seelen-ui/lib/types";
import { CollisionPriority } from "@dnd-kit/abstract";
import { useDroppable } from "@dnd-kit/react";
import { FileIcon, Icon } from "libs/ui/react/components/Icon/index.tsx";
import { cx } from "libs/ui/react/utils/styling.ts";
import { Popover } from "antd";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { AppOrFileWegItem, FolderWegItem } from "../../shared/types.ts";

import { $dock_state_actions } from "../../shared/state/items.ts";
import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";
import { launchItem } from "./UserApplicationContextMenu.tsx";

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
  const id = pendingFolderId;
  pendingFolderId = null;

  if (key === "delete_folder") {
    $dock_state_actions.deleteFolder(id);
  } else if (key.startsWith("color::")) {
    const color = key.slice(7) || null;
    $dock_state_actions.changeFolderColor(id, color);
  }
});

function getPopoverPlacement(position: SeelenWegSide) {
  switch (position) {
    case SeelenWegSide.Bottom:
      return "top";
    case SeelenWegSide.Top:
      return "bottom";
    case SeelenWegSide.Left:
      return "right";
    case SeelenWegSide.Right:
      return "left";
    default:
      return "top";
  }
}

export const FolderItem = memo(({ item }: Props) => {
  const { t } = useTranslation();

  const iconColor = item.color ?? "var(--system-accent-color)";

  const { ref: dropRef, isDropTarget } = useDroppable({
    id: `folder-drop:${item.id}`,
    type: "folder-drop",
    accept: WegItemType.AppOrFile,
    data: { folderId: item.id },
    collisionPriority: CollisionPriority.Highest,
  });

  const onContextMenu = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      pendingFolderId = item.id;
      const { alignX, alignY } = getDockContextMenuAlignment($settings.value.position);

      // Compute a fixed position relative to the icon element (not the mouse cursor)
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const dpr = window.devicePixelRatio;
      const sx = window.screenX;
      const sy = window.screenY;
      const position = $settings.value.position;
      let desiredX: number;
      let desiredY: number;
      if (position === "Bottom") {
        desiredX = (sx + rect.left + rect.width / 2) * dpr; // horizontal center of icon
        desiredY = (sy + rect.top) * dpr; // top edge of icon (menu opens above)
      } else if (position === "Top") {
        desiredX = (sx + rect.left + rect.width / 2) * dpr;
        desiredY = (sy + rect.bottom) * dpr; // bottom edge (menu opens below)
      } else if (position === "Left") {
        desiredX = (sx + rect.right) * dpr; // right edge (menu opens to the right)
        desiredY = (sy + rect.top + rect.height / 2) * dpr; // vertical center of icon
      } else {
        desiredX = (sx + rect.left) * dpr; // left edge (menu opens to the left)
        desiredY = (sy + rect.top + rect.height / 2) * dpr;
      }

      invoke(SeelenCommand.TriggerContextMenu, {
        menu: {
          identifier,
          alignX,
          alignY,
          desiredPosition: { x: Math.round(desiredX), y: Math.round(desiredY) },
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

  const folderNode = (
    <div
      ref={dropRef}
      className={cx("weg-item weg-item-folder", {
        "weg-item-folder-drop-target": isDropTarget,
      })}
      onContextMenu={onContextMenu}
    >
      <Icon
        className="weg-item-icon weg-item-folder-icon"
        iconName="BsFolderFill"
        size="100%"
        style={{ color: iconColor }}
      />
      {item.items.length > 0 && <div className="weg-item-folder-count">{item.items.length}</div>}
    </div>
  );

  // No popover to show when the folder is empty.
  if (item.items.length === 0) {
    return folderNode;
  }

  return (
    <Popover
      placement={getPopoverPlacement($settings.value.position)}
      trigger="hover"
      arrow={false}
      getPopupContainer={() => document.getElementById("root") ?? document.body}
      content={
        <div
          className={cx("weg-folder-popover", $settings.value.position.toLowerCase())}
          onMouseMoveCapture={(e) => e.stopPropagation()}
          onContextMenu={(e) => {
            e.stopPropagation();
            e.preventDefault();
          }}
        >
          {item.items.map((entry) => {
            const appItem = { type: WegItemType.AppOrFile, ...entry } as AppOrFileWegItem;
            return (
              <div
                key={entry.id}
                className="weg-folder-popover-item"
                title={entry.displayName}
                onClick={() => launchItem(appItem, false)}
              >
                <FileIcon
                  className="weg-item-icon"
                  path={entry.relaunch?.icon || entry.path}
                  umid={entry.umid}
                />
                <button
                  type="button"
                  className="weg-folder-popover-remove"
                  title={t("folder_item.remove_from_group", "Remove from group")}
                  onClick={(e) => {
                    e.stopPropagation();
                    $dock_state_actions.removeItemFromFolder(item.id, entry.id);
                  }}
                >
                  <Icon iconName="IoClose" />
                </button>
              </div>
            );
          })}
        </div>
      }
    >
      {folderNode}
    </Popover>
  );
});
