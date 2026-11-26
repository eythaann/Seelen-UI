import type { UserAppWindow, WindowIdentity } from "@seelen-ui/lib/types";
import { getMatchingTrayIcons } from "./activateOrLaunch.ts";
import type { AppOrFileWegItem } from "./types.ts";

export function getEndTaskTargets(
  item: Pick<AppOrFileWegItem, "path" | "relaunch">,
  windows: readonly Pick<UserAppWindow, "id">[],
  trayIcons: Parameters<typeof getMatchingTrayIcons>[1],
): WindowIdentity[] {
  const targets = new Map<string, WindowIdentity>();
  // Prefer the tray window when closing the main window would leave the process running.
  const windowIds = [
    ...getMatchingTrayIcons(item, trayIcons).map((icon) => icon.windowHandle),
    ...windows.map((window) => window.id),
  ];
  for (const id of windowIds) {
    if (!id) continue;
    const [, pid, createdAt] = id.split("_");
    if (!pid || pid === "0" || !createdAt || createdAt === "0") continue;
    const processIdentity = `${pid}_${createdAt}`;
    if (!targets.has(processIdentity)) targets.set(processIdentity, id);
  }
  return [...targets.values()];
}
