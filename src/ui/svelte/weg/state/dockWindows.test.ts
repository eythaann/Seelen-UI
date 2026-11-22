import assert from "node:assert/strict";
import { test } from "node:test";
import { type VirtualDesktops, WegTemporalItemsVisibility } from "@seelen-ui/lib/types";
import { selectDockWindows } from "./dockWindows.ts";

function workspace(id: string, windows: number[]) {
  return { id, name: null, icon: null, wallpaper: null, windows };
}

// monitor-a shows ws-a1 (window 1) and hides ws-a2 (window 2)
// monitor-b shows ws-b1 (window 3) and hides ws-b2 (window 4)
const vd = {
  monitors: {
    "monitor-a": {
      workspaces: [[workspace("ws-a1", [1]), workspace("ws-a2", [2])]],
      active_workspace: "ws-a1",
    },
    "monitor-b": {
      workspaces: [[workspace("ws-b1", [3]), workspace("ws-b2", [4])]],
      active_workspace: "ws-b1",
    },
  },
  pinned: [],
  switching: false,
} as unknown as VirtualDesktops;

const windows = [
  { hwnd: 1, monitor: "monitor-a" },
  { hwnd: 2, monitor: "monitor-a" },
  { hwnd: 3, monitor: "monitor-b" },
  { hwnd: 4, monitor: "monitor-b" },
];

const hwnds = (list: { hwnd: number }[]) => list.map((w) => w.hwnd);

test("All shows every window", () => {
  const result = selectDockWindows(windows, WegTemporalItemsVisibility.All, "monitor-a", vd);
  assert.deepEqual(hwnds(result), [1, 2, 3, 4]);
});

test("OnMonitor shows only the windows on the dock's monitor, on any workspace", () => {
  const result = selectDockWindows(windows, WegTemporalItemsVisibility.OnMonitor, "monitor-a", vd);
  assert.deepEqual(hwnds(result), [1, 2]);
});

test("OnWorkspace hides windows on workspaces that are not shown", () => {
  const result = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-a", vd);
  assert.deepEqual(hwnds(result), [1, 3]);
});

test("OnWorkspace follows a workspace switch", () => {
  const switched = structuredClone(vd);
  switched.monitors["monitor-a"]!.active_workspace = "ws-a2";

  const result = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-a", switched);
  assert.deepEqual(hwnds(result), [2, 3]);
});

test("OnWorkspace gives the same result on every monitor's dock", () => {
  const onA = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-a", vd);
  const onB = selectDockWindows(windows, WegTemporalItemsVisibility.OnWorkspace, "monitor-b", vd);
  assert.deepEqual(hwnds(onA), hwnds(onB));
});
