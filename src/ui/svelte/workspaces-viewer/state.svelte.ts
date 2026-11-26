import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import type { Wallpaper } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";
import { monitors } from "libs/ui/svelte/runes/monitors.svelte.ts";
import { PreviewSnapshots } from "libs/ui/svelte/utils/PreviewSnapshots.svelte.ts";

let wallpapers = lazyRune(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);

let windows = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, windows.setByPayload);

const previews = new PreviewSnapshots();
Widget.self.onVisibilityChange((visible) => {
  if (visible) previews.start();
  else previews.stop();
});

await Promise.all([
  monitors.init(),
  wallpapers.init(),
  workspaces.init(),
  windows.init(),
]);

$effect.root(() => {
  $effect(() => {
    Widget.self.setPosition(monitors.desktopRect);
  });
});

class State {
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
