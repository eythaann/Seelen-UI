import { DragDropProvider, DragOverlay } from "@dnd-kit/react";
import { move } from "@dnd-kit/helpers";
import type { Droppable } from "@dnd-kit/abstract";
import { WegItemType, WegPinnedItemsVisibility, WegTemporalItemsVisibility } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";

import { FolderItem } from "../item/infra/FolderItem.tsx";
import { MediaSession } from "../item/infra/MediaSession.tsx";
import { Separator } from "../item/infra/Separator.tsx";
import { StartMenu } from "../item/infra/StartMenu.tsx";
import { UserApplication } from "../item/infra/UserApplication.tsx";

import type { SwItem } from "../shared/types.ts";

import { $dock_state, $dock_state_actions } from "../shared/state/items.ts";
import { DraggableItem } from "./DraggableItem.tsx";
import { ShowDesktopModule } from "../item/infra/ShowDesktop.tsx";
import { $settings } from "../shared/state/settings.ts";
import { $current_monitor } from "../shared/state/system.ts";
import { computed } from "@preact/signals";
import { $interactables, getWindowsForItem } from "../shared/state/windows.ts";
import { TrashBin } from "../item/infra/RecycleBin.tsx";

const visibleItems = computed(() => {
  const { pinnedItemsVisibility, temporalItemsVisibility } = $settings.value;
  const monitor = $current_monitor.value;

  const showPinned = pinnedItemsVisibility === WegPinnedItemsVisibility.Always || monitor.isPrimary;
  const filterByMonitor = temporalItemsVisibility === WegTemporalItemsVisibility.OnMonitor;

  const windows = filterByMonitor
    ? $interactables.value.filter((w) => {
      return w.monitor === monitor.id;
    })
    : $interactables.value;

  return $dock_state.value.items.filter((item) => {
    if (item.type === "Folder") {
      return showPinned;
    }

    if (item.type !== "AppOrFile") {
      return showPinned;
    }

    if (item.pinned && showPinned) {
      return true;
    }

    return getWindowsForItem(item, windows).length > 0;
  });
});

/**
 * Resolves the folder id for a drop target. A folder exposes two overlapping
 * droppables: a dedicated "folder-drop" zone and its own sortable droppable
 * (whose id equals the folder item id). Either may win collision detection,
 * so we accept both as "drop into folder".
 */
function resolveFolderId(target: Droppable | null | undefined): string | undefined {
  if (!target) return undefined;
  if (target.type === "folder-drop") {
    return (target.data as { folderId?: string } | undefined)?.folderId;
  }
  const item = $dock_state.value.items.find((i) => i.id === String(target.id));
  return item?.type === WegItemType.Folder ? item.id : undefined;
}

export function DockItems() {
  const { t } = useTranslation();

  const isEmpty = visibleItems.value.filter((c) => c.type !== WegItemType.Separator).length === 0;

  return (
    <DragDropProvider
      onDragOver={(event) => {
        const { source, target } = event.operation;
        // While dragging an app over a folder, don't reorder; let it nest on drop.
        if (source?.type === WegItemType.AppOrFile && resolveFolderId(target)) {
          return;
        }
        const newItems = move($dock_state.value.items, event);
        $dock_state.value = { ...$dock_state.value, items: newItems };
      }}
      onDragEnd={(event) => {
        const { source, target } = event.operation;
        if (source?.type !== WegItemType.AppOrFile) return;
        const folderId = resolveFolderId(target);
        if (folderId && folderId !== String(source.id)) {
          $dock_state_actions.moveItemToFolder(String(source.id), folderId);
        }
      }}
    >
      <div className="weg-items">
        {isEmpty ? <span className="weg-empty-state-label">{t("weg.empty")}</span> : (
          visibleItems.value.map((item, index) => {
            return (
              <DraggableItem item={item} key={item.id} index={index}>
                {ItemByType(item, false)}
              </DraggableItem>
            );
          })
        )}
      </div>

      <DragOverlay>
        {(source) => {
          const item = visibleItems.value.find((c) => c.id === source.id);
          return item ? ItemByType(item, true) : null;
        }}
      </DragOverlay>
    </DragDropProvider>
  );
}

function ItemByType(item: SwItem, isOverlay: boolean) {
  if (item.type === WegItemType.AppOrFile) {
    return <UserApplication key={item.id} item={item} isOverlay={isOverlay} />;
  }

  if (item.type === WegItemType.Folder) {
    return <FolderItem key={item.id} item={item} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key={item.id} item={item} />;
  }

  if (item.type === WegItemType.ShowDesktop) {
    return <ShowDesktopModule key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Separator) {
    return <Separator key={item.id} item={item} />;
  }

  if (item.type === WegItemType.TrashBin) {
    return <TrashBin key={item.id} item={item} />;
  }

  return null;
}
