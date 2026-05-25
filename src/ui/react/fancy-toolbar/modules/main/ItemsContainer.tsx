import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
import { Fragment } from "preact";
import { memo } from "preact/compat";
import { computed } from "@preact/signals";

import { $plugins } from "../shared/state/items.ts";
import { SortableItem } from "../item/infra/infra.tsx";
import { isEqual } from "lodash";
import { PinnedTrayIcons } from "./PinnedTrayIcons.tsx";

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
  startIndex: number;
}

function GroupComponent({ id, items, startIndex }: Props) {
  return (
    <div
      className={`ft-bar-container ft-bar-${id}`}
    >
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

        return (
          <Fragment key={module.id}>
            {module.id === "@seelen/tb-system-tray" && <PinnedTrayIcons />}
            <SortableItem module={module} index={startIndex + index} />
          </Fragment>
        );
      })}
    </div>
  );
}

export const Group = memo(GroupComponent, (prevProps, nextProps) => {
  return isEqual(prevProps, nextProps);
});
