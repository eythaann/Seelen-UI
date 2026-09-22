import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { AppNotification, NotificationsMode } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
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
