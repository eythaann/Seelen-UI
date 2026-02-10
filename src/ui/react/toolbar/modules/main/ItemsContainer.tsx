import { useDroppable } from "@dnd-kit/core";
import { horizontalListSortingStrategy, SortableContext } from "@dnd-kit/sortable";
import type { ToolbarItem2 } from "@seelen-ui/lib/types";
import { memo, useMemo } from "preact/compat";

import { componentByModule } from "./mappins.tsx";

interface Props {
  id: string;
  items: ToolbarItem2[];
}

function __ItemsDropableContainer({ id, items }: Props) {
  const { setNodeRef } = useDroppable({ id });

  // Memoize item IDs to prevent recalculation on every render
  const itemIds = useMemo(
    () => items.map((item) => (typeof item === "string" ? item : item.id)),
    [items],
  );

  return (
    <div ref={setNodeRef} className={`ft-bar-${id}`}>
      <SortableContext items={itemIds} strategy={horizontalListSortingStrategy}>
        {items.map(componentByModule)}
      </SortableContext>
    </div>
  );
}

export const ItemsDropableContainer = memo(__ItemsDropableContainer, (prevProps, nextProps) => {
  // Only re-render if id changed or items array changed (deep comparison)
  return (
    prevProps.id === nextProps.id &&
    prevProps.items.length === nextProps.items.length &&
    prevProps.items.every((item, idx) => {
      const prevItem = item;
      const nextItem = nextProps.items[idx];
      if (typeof prevItem === "string" && typeof nextItem === "string") {
        return prevItem === nextItem;
      }
      if (typeof prevItem === "object" && typeof nextItem === "object") {
        return prevItem.id === nextItem.id;
      }
      return false;
    })
  );
});
