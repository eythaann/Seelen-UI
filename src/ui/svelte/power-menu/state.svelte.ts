import { invoke, type Rect, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

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

let user = $state(await invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, (e) => {
  user = e.payload;
});

export const state = {
  get monitors() {
    return monitors;
  },
  get desktopRect() {
    return desktopRect;
  },
  get user() {
    return user;
  },
};
