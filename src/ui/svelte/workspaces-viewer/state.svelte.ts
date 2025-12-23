import { invoke, type Rect, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import type { Wallpaper } from "@seelen-ui/lib/types";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { lazyRune } from "libs/ui/svelte/utils";

let monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
await subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

let wallpapers = lazyRune(() => invoke(SeelenCommand.StateGetWallpapers));
await subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);
await wallpapers.init();

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
  async function updateSize(rect: Rect) {
    let webview = Widget.getCurrent().webview;
    await webview.setPosition(new PhysicalPosition(rect.left, rect.top));
    await webview.setSize(
      new PhysicalSize({
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
      }),
    );
  }

  $effect(() => {
    // as this is async we pass the deps as argument to be scoped to this effect
    updateSize(desktopRect);
  });
});

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
await subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);
await workspaces.init();

let windows = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, windows.setByPayload);
await windows.init();

let previews = lazyRune(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
await subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, previews.setByPayload);
await previews.init();

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
