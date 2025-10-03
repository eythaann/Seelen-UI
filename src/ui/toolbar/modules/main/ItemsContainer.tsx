import { useDroppable } from "@dnd-kit/core";
import { horizontalListSortingStrategy, SortableContext } from "@dnd-kit/sortable";
import { ToolbarItem2 } from "@seelen-ui/lib/types";

import { componentByModule } from "./mappins";

interface Props {
  id: string;
  items: ToolbarItem2[];
}

export function ItemsDropableContainer({ id, items }: Props) {
  const { setNodeRef } = useDroppable({ id });

  const itemIds = items.map((item) => typeof item === "string" ? item : item.id);

  return (
    <div ref={setNodeRef} className={`ft-bar-${id}`}>
      <SortableContext
        items={itemIds}
        strategy={horizontalListSortingStrategy}
      >
        {items.map(componentByModule)}
      </SortableContext>
    </div>
  );
}
