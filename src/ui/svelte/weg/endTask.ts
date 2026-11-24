import type { UserAppWindow, WindowIdentity } from "@seelen-ui/lib/types";
import { getMatchingTrayIcons } from "./activateOrLaunch.ts";
import type { AppOrFileWegItem } from "./types.ts";

export function getEndTaskTargets(
  item: Pick<AppOrFileWegItem, "path" | "relaunch">,
  windows: readonly Pick<UserAppWindow, "hwnd">[],
  trayIcons: Parameters<typeof getMatchingTrayIcons>[1],
): WindowIdentity[] {
  const targets = new Map<string, WindowIdentity>();
  // Prefer the tray handle when closing the main window would leave the process running.
  const handles = [
    ...getMatchingTrayIcons(item, trayIcons).map((icon) => icon.windowHandle),
    ...windows.map((window) => window.hwnd),
  ];
  for (const hwnd of handles) {
    if (!hwnd) continue;
    const [, pid, createdAt] = hwnd.split("_");
    if (!pid || pid === "0" || !createdAt || createdAt === "0") continue;
    const processIdentity = `${pid}_${createdAt}`;
    if (!targets.has(processIdentity)) targets.set(processIdentity, hwnd);
  }
  return [...targets.values()];
}
