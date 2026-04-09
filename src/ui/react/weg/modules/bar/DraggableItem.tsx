import { useSortable } from "@dnd-kit/react/sortable";
import { RestrictToHorizontalAxis, RestrictToVerticalAxis } from "@dnd-kit/abstract/modifiers";

import type { PropsWithChildren } from "preact/compat";

import type { SwItem } from "../shared/types.ts";
import { $settings, isHorizontalDock } from "../shared/state/settings.ts";
import { Alignment, SeelenWegSide } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";
import { $interactables, getWindowsForItem } from "../shared/state/windows.ts";

interface Props extends PropsWithChildren {
  item: SwItem;
  index: number;
  ghost?: boolean;
}

export function DraggableItem({ children, item, index, ghost }: Props) {
  const sortable = useSortable({
    id: item.id,
    index,
    modifiers: [isHorizontalDock.value ? RestrictToHorizontalAxis : RestrictToVerticalAxis],
  });

  const { t } = useTranslation();

  let tooltip = undefined;

  switch (item.type) {
    case "AppOrFile": {
      const windows = getWindowsForItem(item, $interactables.value);
      if (windows.length === 0) {
        tooltip = item.displayName;
      }
      break;
    }
    case "Media":
      tooltip = t("media.label");
      break;
    case "StartMenu":
      tooltip = t("start.label");
      break;
    case "ShowDesktop":
      tooltip = t("show_desktop.label");
      break;
    case "TrashBin":
      tooltip = t("trash_bin.label");
      break;
  }

  let tooltipAlingX = Alignment.Center;
  let tooltipAlingY = Alignment.Center;
  switch ($settings.value.position) {
    case SeelenWegSide.Bottom:
      tooltipAlingY = Alignment.End;
      break;
    case SeelenWegSide.Top:
      tooltipAlingY = Alignment.Start;
      break;
    case SeelenWegSide.Left:
      tooltipAlingX = Alignment.Start;
      break;
    case SeelenWegSide.Right:
      tooltipAlingX = Alignment.End;
      break;
  }

  return (
    <div
      ref={sortable.ref}
      style={{ opacity: sortable.isDragging || ghost ? 0.3 : 1 }}
      data-dragging={sortable.isDragging}
      className="weg-item-drag-container"
      // this was added here to avoid need to pass it to all the items types,
      // this avoid the double context menu of dock menu and dock items.
      onContextMenu={item.type === "Separator" ? undefined : (e) => e.stopPropagation()}
      data-tooltip={tooltip}
      data-tooltip-align-x={tooltipAlingX}
      data-tooltip-align-y={tooltipAlingY}
    >
      {children}
    </div>
  );
}
