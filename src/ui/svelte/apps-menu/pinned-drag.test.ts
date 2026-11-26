import assert from "node:assert/strict";
import { test } from "node:test";
import { createItemIndex, getPinnedDragAction } from "./pinned-drag.ts";

const move = {
  sourceId: "source",
  targetId: "target",
  sourceType: "app",
  targetType: "app",
  pointer: { x: 119, y: 110 },
  targetRect: { left: 100, top: 100, width: 20, height: 20 },
};

test("Pinned drag sorts past the target threshold without touching persisted state", () => {
  const index = createItemIndex(["source", "middle", "target"]);
  assert.deepEqual(getPinnedDragAction(move, index, 0.2), {
    type: "reorder",
    sourceIndex: 0,
    targetIndex: 2,
  });
  assert.equal(index.get("source"), 0);
});

test("Pinned drag retains folder activation before the sorting threshold", () => {
  const index = createItemIndex(["source", "target"]);
  assert.deepEqual(
    getPinnedDragAction(
      { ...move, pointer: { x: 110, y: 110 } },
      index,
      0.2,
    ),
    { type: "dropzone", targetId: "target" },
  );
});

test("Pinned drag does not create a folder when dragging a folder onto an app", () => {
  const index = createItemIndex(["source", "target"]);
  assert.deepEqual(
    getPinnedDragAction({ ...move, sourceType: "folder", pointer: { x: 110, y: 110 } }, index, 0.2),
    { type: "none" },
  );
});

test("Pinned drag can preview a reorder at the vertical edge of a target", () => {
  const index = createItemIndex(["source", "middle", "target"]);
  assert.deepEqual(getPinnedDragAction({ ...move, pointer: { x: 110, y: 119 } }, index, 0.2), {
    type: "reorder",
    sourceIndex: 0,
    targetIndex: 2,
  });
});

test("Pinned drag ignores cached targets outside the pointer", () => {
  const index = createItemIndex(["source", "target"]);
  assert.deepEqual(getPinnedDragAction({ ...move, pointer: { x: 121, y: 110 } }, index, 0.2), { type: "none" });
});
