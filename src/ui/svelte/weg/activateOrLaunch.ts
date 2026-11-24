import type { SysTrayIcon } from "@seelen-ui/lib/types";
import type { AppOrFileWegItem } from "./types.ts";

type TrayIcon = Pick<
  SysTrayIcon,
  "sessionId" | "executablePath" | "isVisible" | "windowHandle" | "uid" | "callbackMessage"
>;

export function getMatchingTrayIcons(
  item: Pick<AppOrFileWegItem, "path" | "relaunch">,
  icons: readonly TrayIcon[],
) {
  const path = item.path.toLowerCase();
  const command = item.relaunch?.command.toLowerCase();
  return icons.filter((icon) => {
    const executable = icon.executablePath.toLowerCase();
    // Explorer hosts Windows shell tray icons unrelated to its folder windows.
    if (executable.split(/[\\/]/).at(-1) === "explorer.exe") return false;
    return executable !== "" && (executable === path || executable === command) &&
      icon.isVisible && icon.windowHandle != null && icon.uid != null && icon.callbackMessage != null;
  });
}

export async function activateOrLaunch(
  item: Pick<AppOrFileWegItem, "path" | "relaunch">,
  getTrayIcons: () => Promise<TrayIcon[]>,
  activateTrayIcon: (id: string) => Promise<void>,
  launch: () => Promise<void>,
) {
  if (item.path || item.relaunch?.command) {
    const matches = getMatchingTrayIcons(item, await getTrayIcons());
    if (matches.length === 1) {
      await activateTrayIcon(matches[0]!.sessionId);
      return;
    }
  }

  await launch();
}
