import { invoke, type Rect, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import { locale } from "./i18n/index.ts";
import { writable } from "svelte/store";

let settings = writable(await Settings.getAsync());
Settings.onChange((s) => settings.set(s));
settings.subscribe((settings) => {
  locale.set(settings.language || "en");
});

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

let menuRect = $derived.by(() => {
  let primary = monitors.find((m) => m.isPrimary);
  if (primary) {
    return primary.rect;
  }
  return desktopRect;
});

let user = $state(await invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, (e) => {
  user = e.payload;
});

export type State = typeof state;
export const state = {
  get monitors() {
    return monitors;
  },
  get desktopRect() {
    return desktopRect;
  },
  get menuRect() {
    return menuRect;
  },
  get user() {
    return user;
  },
};
