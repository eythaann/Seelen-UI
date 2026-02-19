import { DragDropProvider, DragOverlay } from "@dnd-kit/react";
import { move } from "@dnd-kit/helpers";
import { useComputed } from "@preact/signals";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useCallback } from "preact/compat";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import { $plugins, $toolbar_state } from "../shared/state/items.ts";
import { $settings } from "../shared/state/mod.ts";
import { Group } from "./ItemsContainer.tsx";
import { Item } from "../item/infra/infra.tsx";
import { $hidden_by_autohide, $lastFocusedOnMonitor, $thereIsMaximizedOnBg } from "../shared/state/windows.ts";
import { ShowDesktopButton } from "./CornerAction.tsx";
import { useMainContextMenu } from "./ContextMenu.tsx";
import { matchIds } from "../shared/utils.ts";

export function FancyToolbar() {
  const $containers = useComputed(() => ({
    left: $toolbar_state.value.left,
    center: $toolbar_state.value.center,
    right: $toolbar_state.value.right,
  }));

  const contextMenuDef = useMainContextMenu();

  const onContextMenu = useCallback(() => {
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: contextMenuDef,
      forwardTo: null,
    });
  }, [contextMenuDef]);

  return (
    <div
      className={cx("ft-bar", $settings.value.position.toLowerCase(), {
        "ft-bar-hidden": $hidden_by_autohide.value,
      })}
      data-there-is-maximized-on-background={$thereIsMaximizedOnBg.value}
      data-focused-is-maximized={!!$lastFocusedOnMonitor.value?.isMaximized}
      data-focused-is-overlay={!!$lastFocusedOnMonitor.value?.isSeelenOverlay}
      onContextMenu={onContextMenu}
    >
      <ShowDesktopButton />
      <BackgroundByLayersV2 prefix="ft-bar" />

      <DragDropProvider
        onDragOver={(event) => {
          let temp = {
            left: $containers.value.left.map((item) => (typeof item === "string" ? item : item.id)),
            center: $containers.value.center.map((item) => typeof item === "string" ? item : item.id),
            right: $containers.value.right.map((item) => typeof item === "string" ? item : item.id),
          };

          let moved = move(temp, event);
          const allItems = [
            ...$toolbar_state.value.left,
            ...$toolbar_state.value.center,
            ...$toolbar_state.value.right,
          ];

          $toolbar_state.value = {
            ...$toolbar_state.value,
            left: moved.left.map((id) => allItems.find((i) => matchIds(i, id))!),
            center: moved.center.map((id) => allItems.find((i) => matchIds(i, id))!),
            right: moved.right.map((id) => allItems.find((i) => matchIds(i, id))!),
          };
        }}
      >
        {Object.entries($containers.value).map(([id, items]) => <Group key={id} id={id} items={items} />)}

        <DragOverlay>
          {(source) => {
            const allItems = [
              ...$toolbar_state.value.left,
              ...$toolbar_state.value.center,
              ...$toolbar_state.value.right,
            ];

            const entry = allItems.find((i) => matchIds(i, source.id as string));
            if (!entry) return null;

            if (typeof entry === "string") {
              const plugin = $plugins.value.find((p) => p.id === entry);
              if (!plugin) return null;
              const module = { ...(plugin.plugin as any), id: entry };
              return <Item module={module} index={0} group="" />;
            }

            return <Item module={entry} index={0} group="" />;
          }}
        </DragOverlay>
      </DragDropProvider>

      <ShowDesktopButton />
    </div>
  );
}
