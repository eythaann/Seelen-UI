import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import { NotificationsMode, type AppNotification } from "@seelen-ui/lib/types";
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

// The notifications mode API only exists on Windows 11. This await is at module top level,
// so letting it reject would prevent the widget from mounting at all (the user only sees a
// blank popup), therefore fall back to `All` when the command is unavailable.
let notificationsMode = lazyRune(async () => {
  try {
    return await invoke(SeelenCommand.GetNotificationsMode);
  } catch (error) {
    console.warn("Notifications mode is not supported on this OS, defaulting to 'All':", error);
    return NotificationsMode.All;
  }
});
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
