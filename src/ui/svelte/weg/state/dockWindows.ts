import { type VirtualDesktops, WegTemporalItemsVisibility } from "@seelen-ui/lib/types";
import { windowsOnActiveWorkspaces } from "libs/ui/svelte/utils/workspaceFilter.ts";

/** Windows that belong to a dock placed on `monitorId`, honoring the temporal items visibility setting. */
export function selectDockWindows<W extends { hwnd: number; monitor: string }>(
  windows: W[],
  visibility: WegTemporalItemsVisibility,
  monitorId: string | null | undefined,
  virtualDesktops: VirtualDesktops,
): W[] {
  switch (visibility) {
    case WegTemporalItemsVisibility.OnMonitor:
      return windows.filter((w) => w.monitor === monitorId);
    case WegTemporalItemsVisibility.OnWorkspace:
      return windowsOnActiveWorkspaces(windows, virtualDesktops);
    default:
      return windows;
  }
}
