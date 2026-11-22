import assert from "node:assert/strict";
import { test } from "node:test";
import type { VirtualDesktops } from "@seelen-ui/lib/types";
import { windowsOnActiveWorkspaces } from "libs/ui/svelte/utils/workspaceFilter.ts";

function workspace(id: string, windows: number[]) {
  return { id, name: null, icon: null, wallpaper: null, windows };
}

function desktops(partial: Partial<VirtualDesktops>): VirtualDesktops {
  return { monitors: {}, pinned: [], switching: false, ...partial } as VirtualDesktops;
}

const win = (hwnd: number) => ({ hwnd });

test("keeps only the windows of the active workspace", () => {
  const vd = desktops({
    monitors: {
      "monitor-a": {
        workspaces: [[workspace("ws-1", [1, 2]), workspace("ws-2", [3])]],
        active_workspace: "ws-1",
      },
    } as unknown as VirtualDesktops["monitors"],
  });

  const result = windowsOnActiveWorkspaces([win(1), win(2), win(3)], vd);

  assert.deepEqual(result.map((w) => w.hwnd), [1, 2]);
});

test("pinned windows stay visible on every workspace", () => {
  const vd = desktops({
    monitors: {
      "monitor-a": {
        workspaces: [[workspace("ws-1", [1]), workspace("ws-2", [3])]],
        active_workspace: "ws-1",
      },
    } as unknown as VirtualDesktops["monitors"],
    pinned: [3],
  });

  const result = windowsOnActiveWorkspaces([win(1), win(3)], vd);

  assert.deepEqual(result.map((w) => w.hwnd), [1, 3]);
});

test("windows not assigned to any workspace are kept", () => {
  const vd = desktops({
    monitors: {
      "monitor-a": {
        workspaces: [[workspace("ws-1", [1]), workspace("ws-2", [3])]],
        active_workspace: "ws-1",
      },
    } as unknown as VirtualDesktops["monitors"],
  });

  const result = windowsOnActiveWorkspaces([win(1), win(3), win(99)], vd);

  assert.deepEqual(result.map((w) => w.hwnd), [1, 99]);
});

test("each monitor shows its own active workspace", () => {
  const vd = desktops({
    monitors: {
      "monitor-a": {
        workspaces: [[workspace("a-1", [1]), workspace("a-2", [2])]],
        active_workspace: "a-2",
      },
      "monitor-b": {
        workspaces: [[workspace("b-1", [3]), workspace("b-2", [4])]],
        active_workspace: "b-1",
      },
    } as unknown as VirtualDesktops["monitors"],
  });

  const result = windowsOnActiveWorkspaces([win(1), win(2), win(3), win(4)], vd);

  assert.deepEqual(result.map((w) => w.hwnd), [2, 3]);
});

test("nothing is hidden while the workspaces state is still empty", () => {
  const result = windowsOnActiveWorkspaces([win(1), win(2)], desktops({}));

  assert.deepEqual(result.map((w) => w.hwnd), [1, 2]);
});

test("a monitor without workspace data is skipped", () => {
  const vd = desktops({
    monitors: {
      "monitor-a": undefined,
      "monitor-b": {
        workspaces: [[workspace("b-1", [1]), workspace("b-2", [2])]],
        active_workspace: "b-1",
      },
    } as unknown as VirtualDesktops["monitors"],
  });

  const result = windowsOnActiveWorkspaces([win(1), win(2)], vd);

  assert.deepEqual(result.map((w) => w.hwnd), [1]);
});

test("workspaces in several rows are all considered", () => {
  const vd = desktops({
    monitors: {
      "monitor-a": {
        workspaces: [[workspace("r1-1", [1])], [workspace("r2-1", [2]), workspace("r2-2", [3])]],
        active_workspace: "r2-2",
      },
    } as unknown as VirtualDesktops["monitors"],
  });

  const result = windowsOnActiveWorkspaces([win(1), win(2), win(3)], vd);

  assert.deepEqual(result.map((w) => w.hwnd), [3]);
});
