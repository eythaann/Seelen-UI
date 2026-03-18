import { DragDropProvider, DragOverlay } from "@dnd-kit/react";
import { move } from "@dnd-kit/helpers";
import { WegItemType, WegPinnedItemsVisibility, WegTemporalItemsVisibility } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";

import { MediaSession } from "../item/infra/MediaSession.tsx";
import { Separator } from "../item/infra/Separator.tsx";
import { StartMenu } from "../item/infra/StartMenu.tsx";
import { UserApplication } from "../item/infra/UserApplication.tsx";

import type { SwItem } from "../shared/types.ts";

import { $dock_state } from "../shared/state/items.ts";
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
    if (item.type !== "AppOrFile") {
      return showPinned;
    }

    if (item.pinned && showPinned) {
      return true;
    }

    return getWindowsForItem(item, windows).length > 0;
  });
});

export function DockItems() {
  const { t } = useTranslation();

  const isEmpty = visibleItems.value.filter((c) => c.type !== WegItemType.Separator).length === 0;

  return (
    <DragDropProvider
      onDragOver={(event) => {
        const newItems = move($dock_state.value.items, event);
        $dock_state.value = { ...$dock_state.value, items: newItems };
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
