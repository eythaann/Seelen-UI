import {
  closestCorners,
  DndContext,
  type DragEndEvent,
  type DragOverEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { arrayMove } from "@dnd-kit/sortable";
import { useComputed, useSignal } from "@preact/signals";
import type { ToolbarItem2 } from "@seelen-ui/lib/types";
import { AnimatedDropdown } from "@shared/components/AnimatedWrappers";
import { useWindowFocusChange } from "libs/ui/react/utils/hooks.ts";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useState } from "react";

import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import { $toolbar_state } from "../shared/state/items.ts";
import { $settings } from "../shared/state/mod.ts";
import { matchIds } from "../shared/utils.ts";
import { MainContextMenu } from "./ContextMenu.tsx";
import { ItemsDropableContainer } from "./ItemsContainer.tsx";
import { componentByModule } from "./mappins.tsx";
import { $bar_should_be_hidden, $lastFocusedOnMonitor, $thereIsMaximizedOnBg } from "../shared/state/windows.ts";
import { ShowDesktopButton } from "./CornerAction.tsx";

interface Container {
  id: string;
  items: ToolbarItem2[];
}

export function FancyToolbar() {
  const $dragging_id = useSignal<string | null>(null);
  const $containers = useComputed<Container[]>(() => [
    {
      id: "left",
      items: $toolbar_state.value.left,
    },
    {
      id: "center",
      items: $toolbar_state.value.center,
    },
    {
      id: "right",
      items: $toolbar_state.value.right,
    },
  ]);

  const [openContextMenu, setOpenContextMenu] = useState(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  const pointerSensor = useSensor(PointerSensor, {
    activationConstraint: {
      distance: 5,
    },
  });
  const sensors = useSensors(pointerSensor);

  function findContainer(id: string): Container | undefined {
    if (["left", "center", "right"].includes(id)) {
      return $containers.value.find((c) => c.id === id);
    }
    return $containers.value.find((c) => c.items.some((item) => matchIds(item, id)));
  }

  // this handles the item container change while dragging
  function handleDragOver({ active, over }: DragOverEvent) {
    if (!over) return;

    const activeContainer = findContainer(active.id as string);
    const overContainer = findContainer(over.id as string);

    if (!activeContainer || !overContainer || activeContainer.id === overContainer.id) return;

    const activeItem = activeContainer.items.find((item) => item === active.id);
    if (!activeItem) return;

    const newOverContainerItems = [...overContainer.items];
    const overItemIdx = overContainer.items.findIndex((item) => item === over.id);
    if (overItemIdx !== -1) {
      newOverContainerItems.splice(overItemIdx, 0, activeItem);
    } else {
      newOverContainerItems.push(activeItem);
    }

    $toolbar_state.value = {
      ...$toolbar_state.value,
      [activeContainer.id]: activeContainer.items.filter(
        (item) => !matchIds(item, active.id as string),
      ),
      [overContainer.id]: newOverContainerItems,
    };
  }

  // this will handle the sorting
  function handleDragEnd({ active, over }: DragEndEvent) {
    if (!over || active.id === over.id) {
      return;
    }

    const activeContainer = findContainer(active.id as string);
    const overContainer = findContainer(over.id as string);

    if (!activeContainer || !overContainer || activeContainer.id !== overContainer.id) {
      return;
    }

    const activeIndex = activeContainer.items.findIndex((item) => matchIds(item, active.id as string));
    const overIndex = overContainer.items.findIndex((item) => matchIds(item, over.id as string));

    if (activeIndex !== -1 && overIndex !== -1) {
      const newItems = arrayMove(activeContainer.items, activeIndex, overIndex);
      $toolbar_state.value = {
        ...$toolbar_state.value,
        [activeContainer.id]: newItems,
      };
    }
  }

  const activeContainer = $dragging_id.value ? findContainer($dragging_id.value) : undefined;
  const draggingItem = activeContainer?.items.find((item) => matchIds(item, $dragging_id.value!));

  return (
    <AnimatedDropdown
      animationDescription={{
        openAnimationName: "ft-bar-context-menu-open",
        closeAnimationName: "ft-bar-context-menu-close",
      }}
      trigger={["contextMenu"]}
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      popupRender={() => <MainContextMenu />}
    >
      <div
        className={cx("ft-bar", $settings.value.position.toLowerCase(), {
          "ft-bar-hidden": $bar_should_be_hidden.value,
        })}
        data-there-is-maximized-on-background={$thereIsMaximizedOnBg.value}
        data-focused-is-maximized={!!$lastFocusedOnMonitor.value?.isMaximized}
        data-focused-is-overlay={!!$lastFocusedOnMonitor.value?.isSeelenOverlay}
      >
        <ShowDesktopButton />
        <BackgroundByLayersV2 prefix="ft-bar" />

        <DndContext
          collisionDetection={closestCorners}
          onDragStart={({ active }: DragStartEvent) => {
            $dragging_id.value = active.id as string;
          }}
          onDragOver={handleDragOver}
          onDragEnd={(e: DragEndEvent) => {
            handleDragEnd(e);
            $dragging_id.value = null;
          }}
          sensors={sensors}
        >
          {$containers.value.map(({ id, items }) => <ItemsDropableContainer key={id} id={id} items={items} />)}
          <DragOverlay>{draggingItem && componentByModule(draggingItem)}</DragOverlay>
        </DndContext>

        <ShowDesktopButton />
      </div>
    </AnimatedDropdown>
  );
}
