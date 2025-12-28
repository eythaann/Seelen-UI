import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { cx } from "libs/ui/react/utils/styling.ts";
import type { HTMLAttributes, PropsWithChildren } from "preact/compat";

import type { SwItem } from "../shared/types.ts";

interface Props extends PropsWithChildren {
  item: SwItem;
}

export function DraggableItem({ children, item }: Props) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: item.id,
    animateLayoutChanges: () => false,
    disabled: item.type === "Separator",
  });

  return (
    <div
      ref={setNodeRef}
      {...(attributes as HTMLAttributes<HTMLDivElement>)}
      {...listeners}
      style={{
        transform: CSS.Translate.toString(transform),
        transition,
        opacity: isDragging ? 0.3 : 1,
      }}
      className={cx("weg-item-drag-container", {
        dragging: isDragging,
      })}
      // this was added here to avoid need to pass it to all the items types,
      // this avoid the double context menu of dock menu and dock items.
      onContextMenu={item.type === "Separator" ? undefined : (e) => e.stopPropagation()}
    >
      {children}
    </div>
  );
}
