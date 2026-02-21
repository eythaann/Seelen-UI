import { invoke, type Rect, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import type { Wallpaper } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

let monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

let wallpapers = lazyRune(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);

let windows = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, windows.setByPayload);

let previews = lazyRune(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, previews.setByPayload);

await Promise.all([
  monitors.init(),
  wallpapers.init(),
  workspaces.init(),
  windows.init(),
  previews.init(),
]);

let desktopRect = $derived.by(() => {
  let rect: Rect = { top: 0, left: 0, right: 0, bottom: 0 };
  for (const monitor of monitors.value) {
    rect.left = Math.min(rect.left, monitor.rect.left);
    rect.top = Math.min(rect.top, monitor.rect.top);
    rect.right = Math.max(rect.right, monitor.rect.right);
    rect.bottom = Math.max(rect.bottom, monitor.rect.bottom);
  }
  return rect;
});

const relativeMonitors = $derived.by(() => {
  return monitors.value.map((monitor) => {
    return {
      ...monitor,
      rect: {
        ...monitor.rect,
        left: monitor.rect.left - desktopRect.left,
        top: monitor.rect.top - desktopRect.top,
        right: monitor.rect.right - desktopRect.left,
        bottom: monitor.rect.bottom - desktopRect.top,
      },
    };
  });
});

$effect.root(() => {
  $effect(() => {
    Widget.self.setPosition(desktopRect);
  });
});

class State {
  get monitors() {
    return relativeMonitors;
  }
  get workspaces() {
    return workspaces.value;
  }
  get windows() {
    return windows.value;
  }
  get previews() {
    return previews.value;
  }
  get wallpapers() {
    return wallpapers.value;
  }

  findWallpaper(wallpaperId: string | undefined | null): Wallpaper | undefined {
    if (!wallpaperId) return undefined;
    return this.wallpapers.find((w) => w.id === wallpaperId);
  }
}

export const state = new State();
