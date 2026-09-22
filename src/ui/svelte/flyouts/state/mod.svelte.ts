import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { NotificationsMode } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

let shortcutsPaused = $state<boolean | null>(null);
subscribe(SeelenEvent.ShortcutsPaused, (e) => {
  shortcutsPaused = e.payload;
});

let mediaDevices = lazyRune(() => invoke(SeelenCommand.GetMediaDevices));
subscribe(SeelenEvent.MediaDevices, mediaDevices.setByPayload);

let mediaPlaying = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, mediaPlaying.setByPayload);

let brightness = lazyRune(() => invoke(SeelenCommand.GetAllMonitorsBrightness));
subscribe(SeelenEvent.SystemMonitorsBrightnessChanged, brightness.setByPayload);

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);

let notifications = lazyRune(() => invoke(SeelenCommand.GetNotifications));
subscribe(SeelenEvent.Notifications, notifications.setByPayload);

// Same as the notifications widget: the mode API is Windows 11 only, and this promise sits
// inside the `Promise.all` below, so a rejection would prevent the whole flyouts widget
// from mounting.
let notificationsMode = lazyRune(async () => {
  try {
    return await invoke(SeelenCommand.GetNotificationsMode);
  } catch (error) {
    console.warn("Notifications mode is not supported on this OS, defaulting to 'All':", error);
    return NotificationsMode.All;
  }
});
subscribe(SeelenEvent.NotificationsModeChanged, notificationsMode.setByPayload);

await Promise.all([
  mediaDevices.init(),
  mediaPlaying.init(),
  brightness.init(),
  workspaces.init(),
  notifications.init(),
  notificationsMode.init(),
]);

export const state = {
  get mediaInputs() {
    return mediaDevices.value[0];
  },
  get mediaOutputs() {
    return mediaDevices.value[1];
  },
  get mediaPlaying() {
    return mediaPlaying.value;
  },
  get brightness() {
    return brightness.value[0] || null;
  },
  get workspaces() {
    return workspaces.value;
  },
  get notifications() {
    return notifications.value;
  },
  get notificationsMode(): NotificationsMode {
    return notificationsMode.value;
  },
  get shortcutsPaused() {
    return shortcutsPaused;
  },
};
