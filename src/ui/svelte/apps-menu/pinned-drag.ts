export interface DragRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface PinnedDragMove {
  sourceId: string;
  targetId: string;
  sourceType: string | undefined;
  targetType: string | undefined;
  pointer: { x: number; y: number };
  targetRect: DragRect;
}

export type PinnedDragAction =
  | { type: "none" }
  | { type: "reorder"; sourceIndex: number; targetIndex: number }
  | { type: "dropzone"; targetId: string };

export function createItemIndex(itemIds: readonly string[]): ReadonlyMap<string, number> {
  return new Map(itemIds.map((itemId, index) => [itemId, index]));
}

export function getPinnedDragAction(
  move: PinnedDragMove,
  index: ReadonlyMap<string, number>,
  positionThreshold: number,
): PinnedDragAction {
  const { targetRect, pointer } = move;
  if (move.sourceId === move.targetId || targetRect.width === 0 || targetRect.height === 0) {
    return { type: "none" };
  }

  const sourceIndex = index.get(move.sourceId);
  const targetIndex = index.get(move.targetId);
  if (sourceIndex === undefined || targetIndex === undefined) {
    return { type: "none" };
  }

  const x = (pointer.x - targetRect.left) / targetRect.width;
  const y = (pointer.y - targetRect.top) / targetRect.height;
  if (x < 0 || x > 1 || y < 0 || y > 1) return { type: "none" };

  const shouldSort = (sourceIndex > targetIndex && (x < positionThreshold || y < positionThreshold)) ||
    (sourceIndex < targetIndex && (x > 1 - positionThreshold || y > 1 - positionThreshold));

  if (shouldSort) {
    return { type: "reorder", sourceIndex, targetIndex };
  }
  if (move.sourceType === "folder" && move.targetType === "app") {
    return { type: "none" };
  }
  return { type: "dropzone", targetId: move.targetId };
}
