import { arrayMove } from "./utils.ts";

interface PositionedItem {
  slot?: number;
}

// Limit extra empty cells when loading malformed or hand-edited favorites.
const MAX_SAVED_SLOT = 4095;

export function getPinnedGrid<T extends PositionedItem>(items: readonly T[]): Array<T | null> {
  const cells: Array<T | null> = [];
  for (const item of items) {
    const slot = item.slot;
    if (slot !== undefined && Number.isInteger(slot) && slot <= MAX_SAVED_SLOT) {
      while (cells.length < slot) cells.push(null);
    }
    cells.push(item);
  }
  return cells;
}

export function movePinnedItemToEmptySlot<T extends PositionedItem>(
  items: T[],
  sourceIndex: number,
  slot: number,
): T[] {
  const item = items[sourceIndex];
  if (!item || !Number.isInteger(slot) || slot < 0 || slot > MAX_SAVED_SLOT) return items;

  const cells = getPinnedGrid(items);
  if (cells[slot]) return items;

  const sourceSlot = cells.indexOf(item);
  cells[sourceSlot] = null;
  while (cells.length < slot) cells.push(null);
  cells[slot] = item;
  return cells.flatMap((cell, index) => cell ? [{ ...cell, slot: index }] : []);
}

export function reorderPinnedItems<T extends PositionedItem>(
  items: T[],
  sourceIndex: number,
  targetIndex: number,
): T[] {
  const reordered = arrayMove(items, sourceIndex, targetIndex);
  if (!items.some((item) => item.slot !== undefined)) return reordered;

  let index = 0;
  return getPinnedGrid(items).flatMap((cell, slot) => cell ? [{ ...reordered[index++]!, slot }] : []);
}
