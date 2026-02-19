export interface DndContainer<T> {
  id: T;
  items: T[];
}

function getContainerIdx<T>(
  id: T,
  containers: DndContainer<T>[],
) {
  return containers.findIndex((c) => c.id === id || (c.items as unknown[]).includes(id));
}

export function genericHandleDragOver<T extends string>(
  event: any,
  containers: DndContainer<T>[],
  onChange: (newState: DndContainer<T>[]) => void,
) {
  const { source, target } = event.operation;
  if (!target) return;

  const activeContainerIdx = getContainerIdx(source.id as T, containers);
  const overContainerIdx = getContainerIdx(target.id as T, containers);
  if (activeContainerIdx === -1 || overContainerIdx === -1) return; // container not found

  const activeContainer = containers.at(activeContainerIdx)!;
  const overContainer = containers.at(overContainerIdx)!;
  if (activeContainer.id === overContainer.id) return; // moving within the same container (not changing containers)

  const activeItem = activeContainer.items.find((item) => item === source.id);
  if (!activeItem) return;

  const newOverContainerItems = [...overContainer.items];
  const overItemIdx = overContainer.items.findIndex((item) => item === target.id);
  if (overItemIdx !== -1) {
    newOverContainerItems.splice(overItemIdx, 0, activeItem);
  } else {
    newOverContainerItems.push(activeItem);
  }

  const newActiveContainerItems = activeContainer.items.filter((item) => item !== source.id);

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
