import { invoke, type Rect, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";

let monitors = $state(await invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, (e) => {
  monitors = e.payload;
});

let desktopRect = $derived.by(() => {
  let rect: Rect = { top: 0, left: 0, right: 0, bottom: 0 };
  for (const monitor of monitors) {
    rect.left = Math.min(rect.left, monitor.rect.left);
    rect.top = Math.min(rect.top, monitor.rect.top);
    rect.right = Math.max(rect.right, monitor.rect.right);
    rect.bottom = Math.max(rect.bottom, monitor.rect.bottom);
  }
  return rect;
});

$effect.root(() => {
  $effect(() => {
    let webview = Widget.getCurrent().webview;
    webview.setPosition(new PhysicalPosition(desktopRect.left, desktopRect.top));
    webview.setSize(
      new PhysicalSize({
        width: desktopRect.right - desktopRect.left,
        height: desktopRect.bottom - desktopRect.top,
      }),
    );
  });
});

let workspaces = $state(await invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, (e) => {
  workspaces = e.payload;
});

let windows = $state(await invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, (e) => {
  windows = e.payload;
});

class State {
  get monitors() {
    return monitors;
  }
  get workspaces() {
    return workspaces;
  }
  get windows() {
    return windows;
  }
}

export const state = new State();
