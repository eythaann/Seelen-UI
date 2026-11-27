import type { VirtualDesktops, WindowIdentity } from "@seelen-ui/lib/types";
import { windowsOnActiveWorkspaces } from "libs/ui/svelte/utils/workspaceFilter.ts";

export interface SwitcherFilter {
  onlyOnActiveMonitor: boolean;
  onlyOnActiveWorkspace?: boolean;
  /** monitor where the switcher was triggered, null when unknown */
  activeMonitorId: string | null;
  virtualDesktops: VirtualDesktops;
}

/** Windows shown in the task switcher, optionally restricted to the active monitor and/or workspace. */
export function filterSwitcherWindows<W extends { id: WindowIdentity; monitor: string }>(
  windows: W[],
  filter: SwitcherFilter,
): W[] {
  let result = windows;
  if (filter.onlyOnActiveWorkspace) {
    result = windowsOnActiveWorkspaces(result, filter.virtualDesktops);
  }
  if (filter.onlyOnActiveMonitor && filter.activeMonitorId) {
    result = result.filter((w) => w.monitor === filter.activeMonitorId);
  }
  return result;
}
