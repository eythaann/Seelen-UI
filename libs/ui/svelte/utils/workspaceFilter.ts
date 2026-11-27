import type { VirtualDesktops, WindowIdentity } from "@seelen-ui/lib/types";

/**
 * Keeps the windows that are on a workspace currently shown on its monitor, plus the
 * pinned windows (pinned windows stay visible on every workspace).
 *
 * A window that is not assigned to any workspace yet is kept too, so a window never
 * goes missing (from the dock, task switcher...) just because the workspace state hasn't caught up.
 */
export function windowsOnActiveWorkspaces<W extends { id: WindowIdentity }>(
  windows: W[],
  virtualDesktops: VirtualDesktops,
): W[] {
  const assigned = new Set<WindowIdentity>();
  const visible = new Set<WindowIdentity>(virtualDesktops.pinned);

  for (const monitor of Object.values(virtualDesktops.monitors)) {
    if (!monitor) {
      continue;
    }
    for (const workspace of monitor.workspaces.flat()) {
      const isActive = workspace.id === monitor.active_workspace;
      for (const id of workspace.windows) {
        assigned.add(id);
        if (isActive) {
          visible.add(id);
        }
      }
    }
  }

  return windows.filter((w) => visible.has(w.id) || !assigned.has(w.id));
}
