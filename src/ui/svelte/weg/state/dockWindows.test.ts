import assert from "node:assert/strict";
import { test } from "node:test";
import { type VirtualDesktops, WegTemporalItemsVisibility } from "@seelen-ui/lib/types";
import { selectDockWindows } from "./dockWindows.ts";

function workspace(id: string, windows: string[]) {
  return { id, name: null, icon: null, wallpaper: null, windows };
}

// monitor-a shows ws-a1 (window 1) and hides ws-a2 (window 2)
// monitor-b shows ws-b1 (window 3) and hides ws-b2 (window 4)
const vd = {
  monitors: {
    "monitor-a": {
      workspaces: [[workspace("ws-a1", ["1"]), workspace("ws-a2", ["2"])]],
      active_workspace: "ws-a1",
    },
    "monitor-b": {
      workspaces: [[workspace("ws-b1", ["3"]), workspace("ws-b2", ["4"])]],
      active_workspace: "ws-b1",
    },
  },
  pinned: [],
  switching: false,
} as unknown as VirtualDesktops;

const windows = [
  { id: "1", monitor: "monitor-a" },
  { id: "2", monitor: "monitor-a" },
  { id: "3", monitor: "monitor-b" },
  { id: "4", monitor: "monitor-b" },
];

const ids = (list: { id: string }[]) => list.map((w) => w.id);

test("All shows every window", () => {
  const result = selectDockWindows(windows, WegTemporalItemsVisibility.All, "monitor-a", vd);
  assert.deepEqual(ids(result), ["1", "2", "3", "4"]);
});

test("OnMonitor shows only the windows on the dock's monitor, on any workspace", () => {
  const result = selectDockWindows(windows, WegTemporalItemsVisibility.OnMonitor, "monitor-a", vd);
  assert.deepEqual(ids(result), ["1", "2"]);
});

test("OnWorkspace shows only the active workspace of the dock's monitor", () => {
  const onA = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-a", vd);
  const onB = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-b", vd);
  assert.deepEqual(ids(onA), ["1"]);
  assert.deepEqual(ids(onB), ["3"]);
});

test("OnWorkspace follows a workspace switch", () => {
  const switched = structuredClone(vd);
  switched.monitors["monitor-a"]!.active_workspace = "ws-a2";

  const result = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-a", switched);
  assert.deepEqual(ids(result), ["2"]);
});

test("OnWorkspace keeps pinned windows of the dock's monitor only", () => {
  const pinned = { ...vd, pinned: ["2", "4"] };

  const result = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-a", pinned);
  assert.deepEqual(ids(result), ["1", "2"]);
});
