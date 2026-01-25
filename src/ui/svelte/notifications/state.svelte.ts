import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { AppNotification } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { writable } from "svelte/store";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let widget = Widget.getCurrent();
let webview = widget.webview;

let settings = writable(await Settings.getAsync());
Settings.onChange((s) => settings.set(s));
settings.subscribe((settings) => {
  locale.set(settings.language || "en");
});

let notifications = lazyRune(() => invoke(SeelenCommand.GetNotifications));
await subscribe(SeelenEvent.Notifications, notifications.setByPayload);
await notifications.init();

webview.onFocusChanged((e) => {
  if (!e.payload) {
    webview.hide();
  }
});

class State {
  get notifications(): AppNotification[] {
    return notifications.value;
  }
}

export const globalState = new State();
