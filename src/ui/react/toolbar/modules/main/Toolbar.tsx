import { DragDropProvider, DragOverlay } from "@dnd-kit/react";
import { KeyboardSensor, PointerSensor } from "@dnd-kit/dom";
import { move } from "@dnd-kit/helpers";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useCallback } from "preact/compat";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import {
  $plugins,
  $toolbar_state,
  HARDCODED_SEPARATOR_LEFT,
  HARDCODED_SEPARATOR_RIGHT,
} from "../shared/state/items.ts";
import { $settings } from "../shared/state/mod.ts";
import { Alignment, FancyToolbarSide } from "@seelen-ui/lib/types";
import { Group } from "./ItemsContainer.tsx";
import { Item } from "../item/infra/infra.tsx";
import { $hidden_by_autohide, $lastFocusedOnMonitor, $thereIsMaximizedOnBg } from "../shared/state/windows.ts";
import { ShowDesktopButton } from "./CornerAction.tsx";
import { useMainContextMenu } from "./ContextMenu.tsx";
import { matchIds } from "../shared/utils.ts";
import { useComputed } from "@preact/signals";

// Allow dragging from buttons and other interactive elements inside items.
// The distance activation constraint (5px) still prevents unintentional drags on click.
const dndSensors = [
  PointerSensor.configure({ preventActivation: () => false }),
  KeyboardSensor,
];

export function FancyToolbar() {
  const splittedItems = useComputed(() => {
    const items = $toolbar_state.value.items;
    const idx1 = items.findIndex(
      (i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_LEFT.id,
    );
    const idx2 = items.findIndex(
      (i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_RIGHT.id,
    );

    // center includes the separator items
    return {
      left: items.slice(0, idx1),
      center: items.slice(idx1, idx2 + 1),
      right: items.slice(idx2 + 1),
    };
  });

  const contextMenuDef = useMainContextMenu();

  const onContextMenu = useCallback(() => {
    const alignY = $settings.value.position === FancyToolbarSide.Bottom ? Alignment.End : Alignment.Start;
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...contextMenuDef, alignX: Alignment.Center, alignY },
      forwardTo: null,
    });
  }, [contextMenuDef, $settings.value.position]);

  return (
    <div
      className={cx("ft-bar", $settings.value.position.toLowerCase(), {
        "ft-bar-hidden": $hidden_by_autohide.value,
      })}
      data-has-margin={!!$settings.value.margin}
      data-there-is-maximized-on-background={$thereIsMaximizedOnBg.value}
      data-focused-is-maximized={!!$lastFocusedOnMonitor.value?.isMaximized}
      data-focused-is-overlay={!!$lastFocusedOnMonitor.value?.isSeelenOverlay}
      onContextMenu={onContextMenu}
    >
      <ShowDesktopButton />
      <BackgroundByLayersV2 />

      <DragDropProvider
        sensors={dndSensors}
        onDragOver={(event) => {
          const temp = $toolbar_state.value.items.map((item) => typeof item === "string" ? item : item.id);
          const newItems = move(temp, event);

          $toolbar_state.value = {
            isReorderDisabled: $toolbar_state.value.isReorderDisabled,
            items: newItems.map((id) => $toolbar_state.value.items.find((i) => matchIds(i, id))!),
          };
        }}
      >
        <Group id="left" items={splittedItems.value.left} startIndex={0} />
        <Group
          id="center"
          items={splittedItems.value.center}
          startIndex={splittedItems.value.left.length}
        />
        <Group
          id="right"
          items={splittedItems.value.right}
          startIndex={splittedItems.value.left.length + splittedItems.value.center.length}
        />

        <DragOverlay>
          {(source) => {
            const allItems = $toolbar_state.value.items;
            const entry = allItems.find((i) => matchIds(i, source.id as string));
            if (!entry) return null;

            if (typeof entry === "string") {
              const plugin = $plugins.value.find((p) => p.id === entry);
              if (!plugin) return null;
              const module = { ...(plugin.plugin as any), id: entry };
              return <Item module={module} index={0} />;
            }

            return <Item module={entry} index={0} />;
          }}
        </DragOverlay>
      </DragDropProvider>

      <ShowDesktopButton />
    </div>
  );
}
