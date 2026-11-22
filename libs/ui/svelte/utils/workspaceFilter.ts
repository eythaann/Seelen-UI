import type { VirtualDesktops } from "@seelen-ui/lib/types";

/**
 * Keeps the windows that are on a workspace currently shown on its monitor, plus the
 * pinned windows (pinned windows stay visible on every workspace).
 *
 * A window that is not assigned to any workspace yet is kept too, so a window never
 * goes missing (from the dock, task switcher...) just because the workspace state hasn't caught up.
 */
export function windowsOnActiveWorkspaces<W extends { hwnd: number }>(
  windows: W[],
  virtualDesktops: VirtualDesktops,
): W[] {
  const assigned = new Set<number>();
  const visible = new Set<number>(virtualDesktops.pinned);

  for (const monitor of Object.values(virtualDesktops.monitors)) {
    if (!monitor) {
      continue;
    }
    for (const workspace of monitor.workspaces.flat()) {
      const isActive = workspace.id === monitor.active_workspace;
      for (const hwnd of workspace.windows) {
        assigned.add(hwnd);
        if (isActive) {
          visible.add(hwnd);
        }
      }
    }
  }

  return windows.filter((w) => visible.has(w.hwnd) || !assigned.has(w.hwnd));
}
