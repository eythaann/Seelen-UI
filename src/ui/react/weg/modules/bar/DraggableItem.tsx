import { useSortable } from "@dnd-kit/react/sortable";
import { RestrictToHorizontalAxis, RestrictToVerticalAxis } from "@dnd-kit/abstract/modifiers";

import type { PropsWithChildren } from "preact/compat";

import type { SwItem } from "../shared/types.ts";
import { isHorizontalDock } from "../shared/state/settings.ts";

interface Props extends PropsWithChildren {
  item: SwItem;
  index: number;
}

export function DraggableItem({ children, item, index }: Props) {
  const sortable = useSortable({
    id: item.id,
    index,
    modifiers: [isHorizontalDock.value ? RestrictToHorizontalAxis : RestrictToVerticalAxis],
  });

  return (
    <div
      ref={sortable.ref}
      style={{ opacity: sortable.isDragging ? 0.3 : 1 }}
      data-dragging={sortable.isDragging}
      className="weg-item-drag-container"
      // this was added here to avoid need to pass it to all the items types,
      // this avoid the double context menu of dock menu and dock items.
      onContextMenu={item.type === "Separator" ? undefined : (e) => e.stopPropagation()}
    >
      {children}
    </div>
  );
}
