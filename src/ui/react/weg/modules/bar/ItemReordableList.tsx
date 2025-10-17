import {
  closestCorners,
  DndContext,
  type DragEndEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  arrayMove,
  horizontalListSortingStrategy,
  SortableContext,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { batch, useSignal } from "@preact/signals";
import { WegItemType } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";

import { FileOrFolder } from "../item/infra/File.tsx";
import { MediaSession } from "../item/infra/MediaSession.tsx";
import { Separator } from "../item/infra/Separator.tsx";
import { StartMenu } from "../item/infra/StartMenu.tsx";
import { UserApplication } from "../item/infra/UserApplication.tsx";

import type { SwItem } from "../shared/store/domain.ts";

import { $dock_state } from "../shared/state/items.ts";
import { DraggableItem } from "./DraggableItem.tsx";

export function DockItems({ isHorizontal }: { isHorizontal: boolean }) {
  const $active_id = useSignal<string | null>(null);
  const { t } = useTranslation();

  const pointerSensor = useSensor(PointerSensor, {
    activationConstraint: {
      distance: 5,
    },
  });
  const sensors = useSensors(pointerSensor);

  const isEmpty = $dock_state.value.items.filter((c) => c.type !== WegItemType.Separator)
    .length === 0;

  function handleDragStart(e: DragStartEvent) {
    $active_id.value = e.active.id as string;
  }

  function handleDragEnd(e: DragEndEvent) {
    const { active, over } = e;
    if (!over || active.id === over.id) {
      $active_id.value = null;
      return;
    }

    const originalPos = $dock_state.value.items.findIndex((c) => c.id === active.id);
    const newPos = $dock_state.value.items.findIndex((c) => c.id === over.id);
    const newItems = arrayMove($dock_state.value.items, originalPos, newPos);

    batch(() => {
      $active_id.value = null;
      $dock_state.value = { ...$dock_state.value, items: newItems };
    });
  }

  const dragginItem = $dock_state.value.items.find((c) => c.id === $active_id.value);
  return (
    <DndContext
      collisionDetection={closestCorners}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      sensors={sensors}
    >
      <div className="weg-items">
        {isEmpty ? <span className="weg-empty-state-label">{t("weg.empty")}</span> : (
          <SortableContext
            items={$dock_state.value.items}
            strategy={isHorizontal ? horizontalListSortingStrategy : verticalListSortingStrategy}
            disabled={$dock_state.value.isReorderDisabled}
          >
            {$dock_state.value.items.map((item) => (
              <DraggableItem item={item}>
                {ItemByType(item, false)}
              </DraggableItem>
            ))}
          </SortableContext>
        )}
        <DragOverlay>
          {dragginItem && ItemByType(dragginItem, true)}
        </DragOverlay>
      </div>
    </DndContext>
  );
}

function ItemByType(item: SwItem, isOverlay: boolean) {
  if (item.type === WegItemType.Pinned) {
    if (item.subtype === "App") {
      return <UserApplication key={item.id} item={item} isOverlay={isOverlay} />;
    }
    return <FileOrFolder key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Temporal) {
    return <UserApplication key={item.id} item={item} isOverlay={isOverlay} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession key={item.id} item={item} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Separator) {
    return <Separator key={item.id} item={item} />;
  }

  return null;
}
