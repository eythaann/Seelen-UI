import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { AppNotification, NotificationsMode } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { writable } from "svelte/store";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let settings = writable(await Settings.getAsync());
Settings.onChange((s) => settings.set(s));
settings.subscribe((settings) => {
  locale.set(settings.language || "en");
});

let notifications = lazyRune(() => invoke(SeelenCommand.GetNotifications));
subscribe(SeelenEvent.Notifications, notifications.setByPayload);
await notifications.init();

let notificationsMode = lazyRune(() => invoke(SeelenCommand.GetNotificationsMode));
subscribe(SeelenEvent.NotificationsModeChanged, notificationsMode.setByPayload);
await notificationsMode.init();

class State {
  get notifications(): AppNotification[] {
    return notifications.value;
  }

  get focusAssistMode(): NotificationsMode {
    return notificationsMode.value;
  }
}

export const globalState = new State();
