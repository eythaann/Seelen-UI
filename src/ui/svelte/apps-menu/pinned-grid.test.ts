import assert from "node:assert/strict";
import { test } from "node:test";
import { getPinnedGrid, movePinnedItemToEmptySlot, reorderPinnedItems } from "./pinned-grid.ts";

interface Pin {
  itemId: string;
  slot?: number;
}

const original: Pin[] = ["a", "b", "c"].map((itemId) => ({ itemId }));
const idsByCell = (items: Pin[]) => getPinnedGrid(items).map((item) => item?.itemId ?? null);

test("old favorites fill the grid in their original order", () => {
  assert.deepEqual(idsByCell(original), ["a", "b", "c"]);
});

test("an empty-cell drop preserves the other slots and round-trips through favorites", () => {
  const moved = movePinnedItemToEmptySlot(original, 0, 5);
  assert.deepEqual(idsByCell(JSON.parse(JSON.stringify(moved))), [null, "b", "c", null, null, "a"]);
  assert.deepEqual(idsByCell(original), ["a", "b", "c"]);
  assert.deepEqual(idsByCell(movePinnedItemToEmptySlot(moved, 2, 0)), ["a", "b", "c"]);
});

test("reordering onto an occupied position preserves the existing empty cells", () => {
  const moved = movePinnedItemToEmptySlot(original, 0, 5);
  assert.deepEqual(idsByCell(reorderPinnedItems(moved, 2, 0)), [null, "a", "b", null, null, "c"]);
  assert.strictEqual(movePinnedItemToEmptySlot(moved, 2, 1), moved);
});
