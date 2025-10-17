import type { DragOverEvent, UniqueIdentifier } from "@dnd-kit/core";

export interface DndContainer<T extends UniqueIdentifier> {
  id: T;
  items: T[];
}

function getContainerIdx(
  id: UniqueIdentifier,
  containers: DndContainer<UniqueIdentifier>[],
) {
  return containers.findIndex((c) => c.id === id || c.items.includes(id));
}

export function genericHandleDragOver<T extends UniqueIdentifier>(
  { active, over }: DragOverEvent,
  containers: DndContainer<T>[],
  onChange: (newState: DndContainer<T>[]) => void,
) {
  if (!over) return;

  const activeContainerIdx = getContainerIdx(active.id as T, containers);
  const overContainerIdx = getContainerIdx(over.id as T, containers);
  if (activeContainerIdx === -1 || overContainerIdx === -1) return; // container not found

  const activeContainer = containers.at(activeContainerIdx)!;
  const overContainer = containers.at(overContainerIdx)!;
  if (activeContainer.id === overContainer.id) return; // moving within the same container (not changing containers)

  const activeItem = activeContainer.items.find((item) => item === active.id);
  if (!activeItem) return;

  const newOverContainerItems = [...overContainer.items];
  const overItemIdx = overContainer.items.findIndex((item) => item === over.id);
  if (overItemIdx !== -1) {
    newOverContainerItems.splice(overItemIdx, 0, activeItem);
  } else {
    newOverContainerItems.push(activeItem);
  }

  const newActiveContainerItems = activeContainer.items.filter((item) => item !== active.id);

  const newState = [...containers];
  newState[activeContainerIdx] = {
    ...activeContainer,
    items: newActiveContainerItems,
  };
  newState[overContainerIdx] = {
    ...overContainer,
    items: newOverContainerItems,
  };

  onChange(newState);
}
