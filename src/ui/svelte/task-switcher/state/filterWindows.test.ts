import assert from "node:assert/strict";
import { test } from "node:test";
import type { VirtualDesktops } from "@seelen-ui/lib/types";
import { filterSwitcherWindows } from "./filterWindows.ts";

function workspace(id: string, windows: number[]) {
  return { id, name: null, icon: null, wallpaper: null, windows };
}

// monitor-a shows ws-a1 (window 1) and hides ws-a2 (window 2)
// monitor-b shows ws-b1 (window 3) and hides ws-b2 (window 4)
const virtualDesktops = {
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

// already sorted by last focus, as the switcher receives them
const windows = [
  { hwnd: 4, monitor: "monitor-b" },
  { hwnd: 1, monitor: "monitor-a" },
  { hwnd: 3, monitor: "monitor-b" },
  { hwnd: 2, monitor: "monitor-a" },
];

const hwnds = (list: { hwnd: number }[]) => list.map((w) => w.hwnd);

test("no filter shows every window", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: false,
    onlyOnActiveWorkspace: false,
    activeMonitorId: "monitor-a",
    virtualDesktops,
  });
  assert.deepEqual(hwnds(result), [4, 1, 3, 2]);
});

test("a config saved before the workspace option existed shows every window", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: false,
    activeMonitorId: "monitor-a",
    virtualDesktops,
  });
  assert.deepEqual(hwnds(result), [4, 1, 3, 2]);
});

test("only on active monitor keeps the windows of that monitor", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: true,
    onlyOnActiveWorkspace: false,
    activeMonitorId: "monitor-a",
    virtualDesktops,
  });
  assert.deepEqual(hwnds(result), [1, 2]);
});

test("only on active monitor is ignored when the monitor is unknown", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: true,
    onlyOnActiveWorkspace: false,
    activeMonitorId: null,
    virtualDesktops,
  });
  assert.deepEqual(hwnds(result), [4, 1, 3, 2]);
});

test("only on active workspace hides windows on hidden workspaces", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: false,
    onlyOnActiveWorkspace: true,
    activeMonitorId: "monitor-a",
    virtualDesktops,
  });
  assert.deepEqual(hwnds(result), [1, 3]);
});

test("both filters keep only the active workspace of the active monitor", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: true,
    onlyOnActiveWorkspace: true,
    activeMonitorId: "monitor-b",
    virtualDesktops,
  });
  assert.deepEqual(hwnds(result), [3]);
});

test("pinned windows stay in the switcher on every workspace", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: false,
    onlyOnActiveWorkspace: true,
    activeMonitorId: "monitor-a",
    virtualDesktops: { ...virtualDesktops, pinned: [2] },
  });
  assert.deepEqual(hwnds(result), [1, 3, 2]);
});

test("focus order is preserved", () => {
  const result = filterSwitcherWindows(windows, {
    onlyOnActiveMonitor: false,
    onlyOnActiveWorkspace: true,
    activeMonitorId: "monitor-a",
    virtualDesktops: { ...virtualDesktops, pinned: [4] },
  });
  assert.deepEqual(hwnds(result), [4, 1, 3]);
});
