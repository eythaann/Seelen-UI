import { useDroppable } from "@dnd-kit/react";
import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
import { memo } from "preact/compat";
import { computed } from "@preact/signals";

import { $plugins } from "../shared/state/items.ts";
import { SortableItem } from "../item/infra/infra.tsx";
import { isEqual } from "lodash";
import { CollisionPriority } from "@dnd-kit/abstract";

const plugins = computed(() => {
  const dict: Record<string, ToolbarItem> = {};
  for (const plugin of $plugins.value) {
    dict[plugin.id] = plugin.plugin as ToolbarItem;
  }
  return dict;
});

interface Props {
  id: string;
  items: ToolbarItem2[];
}

function GroupComponent({ id, items }: Props) {
  const droppable = useDroppable({
    id,
    type: "container",
    accept: "item",
    collisionPriority: CollisionPriority.Low,
  });

  return (
    <div ref={droppable.ref} className={`ft-bar-container ft-bar-${id}`} data-drop-target={droppable.isDropTarget}>
      {items.map((entry, index) => {
        let module: ToolbarItem | undefined;

        if (typeof entry === "string") {
          const cached = plugins.value[entry];
          if (!cached) {
            return null;
          }

          module = { ...cached, id: entry };
        } else {
          module = entry;
        }

        return <SortableItem key={module.id} module={module} index={index} group={id} />;
      })}
    </div>
  );
}

export const Group = memo(GroupComponent, (prevProps, nextProps) => {
  return isEqual(prevProps, nextProps);
});
