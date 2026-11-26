import type { UserAppWindow } from "@seelen-ui/lib/types";
import type { AppOrFileWegItem } from "../types.ts";

export class AppWindowsIndex {
  private readonly byUmid = new Map<string, UserAppWindow[]>();
  private readonly byPath = new Map<string, UserAppWindow[]>();
  private readonly sourceOrder = new Map<string, number>();
  private readonly matches = new WeakMap<AppOrFileWegItem, UserAppWindow[]>();

  constructor(windows: UserAppWindow[]) {
    for (const [index, window] of windows.entries()) {
      this.sourceOrder.set(window.hwnd, index);
      if (window.umid) {
        AppWindowsIndex.add(this.byUmid, window.umid, window);
        continue;
      }
      const path = window.process.path?.toLowerCase();
      if (path) AppWindowsIndex.add(this.byPath, path, window);
    }
  }

  get(item: AppOrFileWegItem): UserAppWindow[] {
    const cached = this.matches.get(item);
    if (cached) return cached;

    const windows = item.umid ? [...(this.byUmid.get(item.umid) ?? [])] : [];
    for (const path of [item.relaunch?.command, item.path]) {
      if (path) windows.push(...(this.byPath.get(path.toLowerCase()) ?? []));
    }
    const uniqueWindows = [...new Map(windows.map((window) => [window.hwnd, window])).values()]
      .sort((a, b) => this.sourceOrder.get(a.hwnd)! - this.sourceOrder.get(b.hwnd)!);
    this.matches.set(item, uniqueWindows);
    return uniqueWindows;
  }

  private static add(
    index: Map<string, UserAppWindow[]>,
    key: string,
    window: UserAppWindow,
  ): void {
    const windows = index.get(key);
    if (windows) windows.push(window);
    else index.set(key, [window]);
  }
}
